use std::net::SocketAddr;
use std::path::PathBuf;

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

use crate::settings::SharedConfig;

mod api;
pub mod models;

pub async fn start_server(host: &str, port: u16, web_ui_dir: PathBuf, config: SharedConfig) {
    let index = web_ui_dir.join("index.html");

    let app = Router::new()
        .nest("/api", api::router(config))
        .fallback_service(ServeDir::new(&web_ui_dir).fallback(ServeFile::new(&index)));

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("invalid host:port");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind");

    println!("Web UI serving at http://{addr}");

    axum::serve(listener, app).await.expect("server error");
}
