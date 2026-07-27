use anyhow::Context;
use code_map_backend::{app, config::AppConfig};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    code_map_backend::telemetry::init();

    let config = AppConfig::from_env();
    let addr = config.socket_addr()?;
    let router = app::build_router(config.clone())?;

    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind backend server on {addr}"))?;

    info!(%addr, "code-map backend listening");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("backend server failed")?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
