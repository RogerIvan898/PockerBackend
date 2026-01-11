use serde::{Serialize, Deserialize};

use crate::models::{Card, PrivateState, PublicGameState};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebSocketEvent {
    Deal { cards: Vec<Card> },
    Shuffle,
    Error { message: String },
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerEvent {
    RoundStarted,
    GameState(PublicGameState),
    PrivateState(PrivateState),
    BlindPosted { seat: usize, amount: u64 },
    Error { message: String },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientEvent {
    Join { player_id: String },
    Action { action: PlayerAction },
}

#[derive(Serialize, Deserialize)]
pub enum PlayerAction {
    Fold,
    Check,
    Call,
    Bet { amount: u64 },
    Raise { amount: u64 },
    AllIn,
}
