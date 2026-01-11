use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::models::{Card, Rank, Suit};

pub fn init_deck() -> Vec<Card> {
    let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
    let ranks = [
        Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
        Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
        Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
    ];

    let mut deck = Vec::new();
    
    for suit in &suits {
        for rank in &ranks {
            deck.push(Card {
                suit: suit.clone(),
                rank: rank.clone(),
            });
        }
    }
    deck
}

pub fn pick_cards(deck: &[Card], n: usize, rng: &mut StdRng) -> Vec<Card> {
    let n = n.min(deck.len());
    deck.choose_multiple(rng, n).cloned().collect()
}
