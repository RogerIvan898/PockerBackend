mod app;
mod web_socket;
mod models;
mod constants;
mod game;

use std::net::SocketAddr;
use tracing_subscriber;
use anyhow::Result;

use crate::constants::server::{SERVER_ADDRESS, SERVER_PORT};
use crate::game::GameManager;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let (manager_tx, broadcaster) = GameManager::start();

    let app = app::create_routes(manager_tx, broadcaster);

    let addr: SocketAddr = format!("{SERVER_ADDRESS}:{SERVER_PORT}").parse()?;
    
    tracing::info!(%addr, "starting server");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("server stopped");
    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.ok();
    tracing::info!("received shutdown signal");
}
