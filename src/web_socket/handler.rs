use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};

use tokio::sync::{mpsc, broadcast, oneshot};
use serde::Deserialize;

use crate::game::{GameCommand, PlayerAction};
use crate::models::ServerEvent;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    manager_tx: mpsc::Sender<GameCommand>,
    broadcaster: broadcast::Sender<ServerEvent>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, manager_tx, broadcaster))
}

async fn handle_socket(
    mut socket: WebSocket,
    manager_tx: mpsc::Sender<GameCommand>,
    broadcaster: broadcast::Sender<ServerEvent>,
) {
    // IMPORTANT: subscribe before sending Join to avoid race where RoundStarted is missed
    let mut events = broadcaster.subscribe();

    let (join_tx, join_rx) = oneshot::channel();
    if manager_tx.send(GameCommand::Join { reply: join_tx }).await.is_err() {
        return;
    }

    let player_id = match join_rx.await {
        Ok(Ok(id)) => id,
        _ => return,
    };

    tracing::info!("[WS] connected {}", player_id);

    loop {
        tokio::select! {
            ev = events.recv() => {
                match ev {
                    Ok(server_event) => {
                        // always forward public events
                        if socket.send(Message::Text(serde_json::to_string(&server_event).unwrap())).await.is_err() {
                            break;
                        }

                        // for RoundStarted, ask actor for private state and send only to this socket
                        if let ServerEvent::RoundStarted = server_event {
                            let (p_tx, p_rx) = oneshot::channel();
                            // request private state from actor
                            let _ = manager_tx.send(GameCommand::GetPrivateState { player_id: player_id.clone(), reply: p_tx }).await;
                            if let Ok(private) = p_rx.await {
                                if private.hand.is_some() {
                                    if socket.send(Message::Text(serde_json::to_string(&ServerEvent::PrivateState(private)).unwrap())).await.is_err() {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // log and continue
                        tracing::warn!("[WS] {} lagged on events", player_id);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }

            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        #[derive(Deserialize)]
                        struct ClientMsg {
                            action: String,
                            amount: Option<u64>,
                        }

                        let Ok(cmd) = serde_json::from_str::<ClientMsg>(&text) else {
                            continue;
                        };

                        let action = match cmd.action.as_str() {
                            "fold" => PlayerAction::Fold,
                            "check" => PlayerAction::Check,
                            "call" => PlayerAction::Call,
                            "bet" => PlayerAction::Bet { amount: cmd.amount.unwrap_or(0) },
                            "raise" => PlayerAction::Raise { amount: cmd.amount.unwrap_or(0) },
                            "allin" => PlayerAction::AllIn,
                            _ => continue,
                        };

                        let (tx, rx) = oneshot::channel();
                        let _ = manager_tx.send(GameCommand::Action { player_id: player_id.clone(), action, reply: tx }).await;
                        let _ = rx.await; // optionally inspect result
                    }

                    Some(Ok(Message::Close(_))) | None => {
                        let _ = manager_tx.send(GameCommand::Disconnect { player_id: player_id.clone() }).await;
                        break;
                    }

                    Some(Ok(_)) => {}
                    Some(Err(_)) => {
                        let _ = manager_tx.send(GameCommand::Disconnect { player_id: player_id.clone() }).await;
                        break;
                    }
                }
            }
        }
    }

    tracing::info!("[WS] disconnected {}", player_id);
}
