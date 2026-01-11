use axum::{Router, routing::get};
use tokio::sync::{mpsc, broadcast};

use crate::domain::{ServerEvent, GameCommand};
use crate::infrastructure::ws_handler;

pub fn create_routes(
    manager_tx: mpsc::Sender<GameCommand>,
    broadcaster: broadcast::Sender<ServerEvent>,
) -> Router {
    Router::new().route(
        "/ws",
        get({
            let manager_tx = manager_tx.clone();
            let broadcaster = broadcaster.clone();
            
            move |ws| ws_handler(ws, manager_tx, broadcaster)
        }),
    )
}
