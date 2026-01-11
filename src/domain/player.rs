use serde::{Serialize, Deserialize};

use crate::{domain::Card};
use crate::shared::INITIAL_HAND_SIZE;

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub seat: usize,
    pub stack: u64,
    pub hand: Option<[Card; 2]>,
    pub status: PlayerStatus,
    pub committed: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PublicPlayer {
    pub id: String,
    pub seat: usize,
    pub stack: u64,
    pub status: PlayerStatus,
    pub committed: u64,
}

#[derive(Serialize)]
pub struct RevealedHand {
    pub seat: usize,
    pub hand: [Card; INITIAL_HAND_SIZE],
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PrivateState {
    pub hand: Option<[Card; INITIAL_HAND_SIZE]>
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlayerStatus {
    Waiting,
    Active,
    Folded,
    AllIn,
}
