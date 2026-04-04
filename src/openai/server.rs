use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::{routing::post, Router};

use crate::settings::SharedConfig;

use super::handlers::chat_completions;
use super::middleware::{auth_middleware, ip_filter_middleware};

pub type SharedApiServer = Arc<Mutex<Option<tokio::task::AbortHandle>>>;

pub async fn start_openai_server(config: SharedConfig, handle: SharedApiServer) {
    // 1. Read port from config (drop lock immediately)
    let port: u16 = {
        let state = config.lock().unwrap();
        state
            .config
            .api_config
            .port
            .parse()
            .unwrap_or(8080)
    };

    // 2. Abort previous server task if one exists
    {
        let mut h = handle.lock().unwrap();
        if let Some(prev) = h.take() {
            prev.abort();
        }
    }

    // 3. Bind TcpListener before spawning so errors surface synchronously
    let addr: SocketAddr = format!("0.0.0.0:{port}").parse().unwrap();
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("OpenAI API: failed to bind on port {port}: {e}");
            return;
        }
    };

    println!("OpenAI API server listening at http://0.0.0.0:{port}");

    // 4. Build router — ip_filter is last layer so it runs first on incoming requests
    let app = Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .layer(axum::middleware::from_fn_with_state(
            config.clone(),
            auth_middleware,
        ))
        .layer(axum::middleware::from_fn_with_state(
            config.clone(),
            ip_filter_middleware,
        ))
        .with_state(config);

    // 5. Spawn and store the abort handle
    let join_handle = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .ok();
    });

    let mut h = handle.lock().unwrap();
    *h = Some(join_handle.abort_handle());
}
