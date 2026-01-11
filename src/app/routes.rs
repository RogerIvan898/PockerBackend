use axum::{Router, routing::get};
use tokio::sync::{mpsc, broadcast};

use crate::game::GameCommand;
use crate::models::ServerEvent;

pub fn create_routes(
    manager_tx: mpsc::Sender<GameCommand>,
    broadcaster: broadcast::Sender<ServerEvent>,
) -> Router {
    Router::new().route(
        "/ws",
        get({
            let manager_tx = manager_tx.clone();
            let broadcaster = broadcaster.clone();
            move |ws| crate::web_socket::ws_handler(ws, manager_tx, broadcaster)
        }),
    )
}
