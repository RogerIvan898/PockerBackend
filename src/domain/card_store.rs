use std::collections::HashMap;
use crate::domain::Card;

pub struct CardStore {
    pub hands: HashMap<String, [Card; 2]>,
    pub deck: Vec<Card>,
}

impl CardStore {
    pub fn new() -> Self {
        Self { hands: Default::default(), deck: Vec::new() }
    }
}
