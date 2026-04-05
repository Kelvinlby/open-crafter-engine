use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::{routing::get, routing::post, Router};

use crate::settings::SharedConfig;

use super::handlers::{chat_completions, list_models, retrieve_model};
use super::middleware::{auth_middleware, ip_filter_middleware};

pub type SharedApiServer = Arc<Mutex<Option<tokio::task::AbortHandle>>>;

/// Infer the bind address from the configured accepted IP range.
/// A loopback-only range (127.x or ::1) → bind to 127.0.0.1.
/// Anything else (LAN, VPN, 0.0.0.0/0) → bind to 0.0.0.0.
fn infer_bind_host(cidr: &str) -> &'static str {
    let Ok(net) = cidr.parse::<ipnet::IpNet>() else {
        return "127.0.0.1";
    };
    if net.network().is_loopback() {
        "127.0.0.1"
    } else {
        "0.0.0.0"
    }
}

pub async fn start_openai_server(config: SharedConfig, handle: SharedApiServer) {
    // 1. Read config (drop lock immediately)
    let (cidr, port): (String, u16) = {
        let state = config.lock().unwrap_or_else(|e| e.into_inner());
        let cidr = state.config.api_config.accepted_ip_range.clone();
        let port = state.config.api_config.port.parse().unwrap_or(8080);
        (cidr, port)
    };

    // Infer bind host from accepted IP range:
    // loopback-only range → 127.0.0.1, anything else → 0.0.0.0
    let host = infer_bind_host(&cidr);

    // 2. Abort previous server task if one exists
    {
        let mut h = handle.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(prev) = h.take() {
            prev.abort();
        }
    }

    // 3. Bind TcpListener before spawning so errors surface synchronously
    let addr: SocketAddr = format!("{host}:{port}").parse().unwrap_or_else(|_| {
        tracing::warn!("OpenAI API: invalid bind address '{}:{}', falling back to 127.0.0.1", host, port);
        format!("127.0.0.1:{port}").parse().unwrap()
    });
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("OpenAI API: failed to bind on {addr}: {e}");
            return;
        }
    };

    tracing::info!("OpenAI API server listening at http://{addr}");

    // 4. Build router — ip_filter is last layer so it runs first on incoming requests
    let app = Router::new()
        .route("/v1/models", get(list_models))
        .route("/v1/models/{model_id}", get(retrieve_model))
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

    let mut h = handle.lock().unwrap_or_else(|e| e.into_inner());
    *h = Some(join_handle.abort_handle());
}
