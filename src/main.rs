mod openai;
mod settings;
mod utils;
mod web;

use std::sync::{Arc, Mutex};

use clap::Parser;

#[derive(Parser)]
#[command(name = "open-crafter-engine")]
struct Args {
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(long, default_value_t = 6121)]
    port: u16,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    let exe_dir = std::env::current_exe()
        .expect("failed to determine executable path")
        .parent()
        .expect("executable has no parent directory")
        .to_path_buf();

    let web_ui_dir = exe_dir.join("web-ui");

    if !web_ui_dir.exists() {
        tracing::error!("Web UI directory not found at: {}", web_ui_dir.display());
        std::process::exit(1);
    }

    let config = settings::load(&exe_dir);

    let openai_handle: openai::SharedApiServer = Arc::new(Mutex::new(None));
    openai::start_openai_server(config.clone(), openai_handle.clone()).await;

    web::start_server(&args.host, args.port, web_ui_dir, config, openai_handle).await;
}
