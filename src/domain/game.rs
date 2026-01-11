use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tokio::sync::{oneshot};

use crate::domain::{Card, PlayerAction, PublicPlayer, PrivateState};
use crate::shared::INITIAL_HAND_SIZE;

#[derive(Clone, Serialize, Deserialize)]
pub struct PublicGameState {
    pub players: Vec<PublicPlayer>,
    pub community_cards: Vec<Card>,
    pub pot: u64,
    pub dealer_seat: usize,
    pub current_turn_seat: Option<usize>,
    pub phase: RoundPhase,
    pub small_blind_amount: u64,
    pub big_blind_amount: u64,
    pub current_bet: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum RoundPhase {
    Waiting,
    Preflop,
    Flop,
    Turn,
    River,
    Showdown,
}

pub struct CardStore {
    pub hands: HashMap<String, [Card; INITIAL_HAND_SIZE]>,
    pub deck: Vec<Card>,
}

pub enum GameCommand {
    Join { reply: oneshot::Sender<Result<String, String>> },
    Action { player_id: String, action: PlayerAction, reply: oneshot::Sender<Result<(), String>> },
    Disconnect { player_id: String },
    GetPrivateState { player_id: String, reply: oneshot::Sender<PrivateState> },
}
