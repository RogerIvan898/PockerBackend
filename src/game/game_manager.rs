use tokio::sync::{mpsc, broadcast};
use uuid::Uuid;
use rand::{rngs::StdRng, SeedableRng};
use rand::seq::SliceRandom;

use crate::models::{
    Card, PlayerStatus, PrivateState, PublicGameState, PublicPlayer,
    Rank, RoundPhase, ServerEvent, Suit,
};

#[derive(Debug)]
pub enum PlayerAction {
    Fold,
    Check,
    Call,
    Bet { amount: u64 },
    Raise { amount: u64 },
    AllIn,
}

pub enum GameCommand {
    Join { reply: tokio::sync::oneshot::Sender<Result<String, String>> },
    Action { player_id: String, action: PlayerAction, reply: tokio::sync::oneshot::Sender<Result<(), String>> },
    Disconnect { player_id: String },
    GetPrivateState { player_id: String, reply: tokio::sync::oneshot::Sender<PrivateState> },
}

pub struct CardStore {
    pub hands: std::collections::HashMap<String, [Card; 2]>,
    pub deck: Vec<Card>,
}

impl CardStore {
    pub fn new() -> Self {
        Self { hands: Default::default(), deck: Vec::new() }
    }
}

pub struct GameManager {
    pub state: PublicGameState,
    pub cards: CardStore,
    pub broadcaster: broadcast::Sender<ServerEvent>,
    rng: StdRng,
}

impl GameManager {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(128);
        Self {
            state: PublicGameState {
                players: vec![],
                community_cards: vec![],
                pot: 0,
                dealer_seat: 0,
                current_turn_seat: None,
                phase: RoundPhase::Waiting,
                small_blind_amount: 10,
                big_blind_amount: 20,
                current_bet: 0,
            },
            cards: CardStore::new(),
            broadcaster: tx,
            rng: StdRng::from_entropy(),
        }
    }

    pub fn start() -> (mpsc::Sender<GameCommand>, broadcast::Sender<ServerEvent>) {
        let (tx_cmd, rx_cmd) = mpsc::channel::<GameCommand>(256);
        let mut manager = GameManager::new();
        let broadcaster = manager.broadcaster.clone();
        tokio::spawn(async move { manager.run(rx_cmd).await; });
        (tx_cmd, broadcaster)
    }

    async fn run(&mut self, mut rx: mpsc::Receiver<GameCommand>) {
        while let Some(cmd) = rx.recv().await {
            match cmd {
                GameCommand::Join { reply } => {
                    let res = self.add_waiting_player();
                    let _ = reply.send(res);
                }
                GameCommand::Action { player_id, action, reply } => {
                    let res = self.handle_action(&player_id, action);
                    let _ = reply.send(res);
                }
                GameCommand::Disconnect { player_id } => {
                    self.handle_disconnect(&player_id);
                }
                GameCommand::GetPrivateState { player_id, reply } => {
                    let private = self.private_state(&player_id);
                    let _ = reply.send(private);
                }
            }
        }
        tracing::info!("GameManager actor exiting (command channel closed)");
    }

    fn add_waiting_player(&mut self) -> Result<String, String> {
        if self.state.players.len() >= 9 {
            return Err("Table full".into());
        }

        let player_id = Uuid::new_v4().to_string();
        let seat = self.state.players.len();

        self.state.players.push(PublicPlayer {
            id: player_id.clone(),
            seat,
            stack: 1000,
            status: PlayerStatus::Waiting,
            committed: 0,
        });

        let _ = self.broadcaster.send(ServerEvent::GameState(self.masked_state()));

        if matches!(self.state.phase, RoundPhase::Waiting) {
            let active_count = self.state.players.iter().filter(|p| p.status == PlayerStatus::Active).count();
            let waiting_count = self.state.players.iter().filter(|p| p.status == PlayerStatus::Waiting).count();
            if active_count + waiting_count >= 2 {
                for p in &mut self.state.players {
                    if p.status == PlayerStatus::Waiting {
                        p.status = PlayerStatus::Active;
                    }
                }
                self.start_new_round();
            }
        }

        Ok(player_id)
    }

    fn handle_action(&mut self, player_id: &str, action: PlayerAction) -> Result<(), String> {
        let p = self.state.players.iter_mut().find(|p| p.id == player_id).ok_or("player not found")?;
        match action {
            PlayerAction::Fold => {
                p.status = PlayerStatus::Waiting;
            }
            PlayerAction::Check | PlayerAction::Call => {}
            PlayerAction::Bet { amount } | PlayerAction::Raise { amount } => {
                let a = amount.min(p.stack);
                p.stack = p.stack.saturating_sub(a);
                p.committed += a;
                self.state.pot += a;
            }
            PlayerAction::AllIn => {
                let a = p.stack;
                p.stack = 0;
                p.committed += a;
                self.state.pot += a;
            }
        }

        let _ = self.broadcaster.send(ServerEvent::GameState(self.masked_state()));
        Ok(())
    }

    fn handle_disconnect(&mut self, player_id: &str) {
        if let Some(p) = self.state.players.iter_mut().find(|p| p.id == player_id) {
            p.status = PlayerStatus::Waiting;
        }
        let _ = self.broadcaster.send(ServerEvent::GameState(self.masked_state()));
    }

    fn private_state(&self, player_id: &str) -> PrivateState {
        let hand = self.cards.hands.get(player_id).cloned();
        PrivateState { hand }
    }

    fn masked_state(&self) -> PublicGameState {
        PublicGameState {
            players: self.state.players.clone(),
            community_cards: self.state.community_cards.clone(),
            pot: self.state.pot,
            dealer_seat: self.state.dealer_seat,
            current_turn_seat: self.state.current_turn_seat,
            phase: self.state.phase.clone(),
            small_blind_amount: self.state.small_blind_amount,
            big_blind_amount: self.state.big_blind_amount,
            current_bet: self.state.current_bet,
        }
    }

    fn init_deck(&mut self) {
        let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
        let ranks = [
            Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
            Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
            Rank::Jack, Rank::Queen, Rank::King, Rank::Ace,
        ];

        self.cards.deck.clear();
        for suit in &suits {
            for rank in &ranks {
                self.cards.deck.push(Card { suit: *suit, rank: *rank });
            }
        }

        self.cards.deck.shuffle(&mut self.rng);
    }

    fn reset_round_state(&mut self) {
        self.init_deck();
        self.state.community_cards.clear();
        self.state.pot = 0;
        for p in &mut self.state.players {
            p.committed = 0;
        }
        self.cards.hands.clear();
    }

    fn deal_hole_cards(&mut self) {
        let active_ids: Vec<String> = self.state.players
            .iter()
            .filter(|p| p.status == PlayerStatus::Active)
            .map(|p| p.id.clone())
            .collect();

        for pid in active_ids {
            let c1 = self.cards.deck.pop().expect("deck empty when dealing hole cards");
            let c2 = self.cards.deck.pop().expect("deck empty when dealing hole cards");
            self.cards.hands.insert(pid, [c1, c2]);
        }
    }

    fn burn(&mut self) {
        let _ = self.cards.deck.pop();
    }

    fn deal_flop(&mut self) {
        self.burn();
        for _ in 0..3 {
            if let Some(c) = self.cards.deck.pop() {
                self.state.community_cards.push(c);
            }
        }
        self.state.phase = RoundPhase::Flop;
        let _ = self.broadcaster.send(ServerEvent::GameState(self.masked_state()));
    }

    fn deal_turn(&mut self) {
        self.burn();
        if let Some(c) = self.cards.deck.pop() {
            self.state.community_cards.push(c);
        }
        self.state.phase = RoundPhase::Turn;
        let _ = self.broadcaster.send(ServerEvent::GameState(self.masked_state()));
    }

    fn deal_river(&mut self) {
        self.burn();
        if let Some(c) = self.cards.deck.pop() {
            self.state.community_cards.push(c);
        }
        self.state.phase = RoundPhase::River;
        let _ = self.broadcaster.send(ServerEvent::GameState(self.masked_state()));
    }

    fn move_dealer(&mut self) {
        if self.state.players.is_empty() { return; }
        self.state.dealer_seat = (self.state.dealer_seat + 1) % self.state.players.len();
    }

    fn post_blinds(&mut self) {
        if self.state.players.len() < 2 { return; }

        let small_blind_seat = self.next_active_seat(self.state.dealer_seat);
        let big_blind_seat = self.next_active_seat(small_blind_seat);

        self.apply_blind(small_blind_seat, self.state.small_blind_amount);
        self.apply_blind(big_blind_seat, self.state.big_blind_amount);

        self.state.current_bet = self.state.big_blind_amount;
    }

    fn apply_blind(&mut self, seat: usize, amount: u64) {
        if seat >= self.state.players.len() { return; }

        let player = &mut self.state.players[seat];
        let blind = amount.min(player.stack);

        player.stack = player.stack.saturating_sub(blind);

        player.committed += blind;
        self.state.pot += blind;

        if player.stack == 0 {
            player.status = PlayerStatus::AllIn;
        }

        let _ = self.broadcaster.send(ServerEvent::BlindPosted { seat, amount: blind });
    }

    fn next_active_seat(&self, from: usize) -> usize {
        if self.state.players.is_empty() { return 0; }
        
        let mut seat = from;

        loop {
            seat = (seat + 1) % self.state.players.len();
            if self.state.players[seat].status == PlayerStatus::Active {
                return seat;
            }
        }
    }

    fn start_new_round(&mut self) {
        let active_players_count = self.state.players
            .iter()
            .filter(|p| p.status == PlayerStatus::Active)
            .count();

        let waiting_players_count = self.state.players
            .iter()
            .filter(|p| p.status == PlayerStatus::Waiting)
            .count();

        if active_players_count + waiting_players_count < 2 {
            return;
        }

        for p in &mut self.state.players {
            if p.status == PlayerStatus::Waiting {
                p.status = PlayerStatus::Active;
            }
        }

        self.reset_round_state();
        self.move_dealer();
        self.post_blinds();
        self.deal_hole_cards();

        self.state.phase = RoundPhase::Preflop;
        self.state.current_turn_seat = Some(self.next_active_seat(self.state.dealer_seat));

        let _ = self.broadcaster.send(ServerEvent::GameState(self.masked_state()));
        let _ = self.broadcaster.send(ServerEvent::RoundStarted);
    }

}
