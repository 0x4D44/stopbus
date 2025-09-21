use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub type CardId = u8;
pub const DECK_SIZE: usize = 52;
pub const PLAYERS: usize = 4;
pub const HAND_SIZE: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageKind {
    Info,
    Alert,
}

#[derive(Debug, Clone)]
pub struct GameEvent {
    pub kind: MessageKind,
    pub text: String,
}

impl GameEvent {
    pub fn info<T: Into<String>>(text: T) -> Self {
        Self {
            kind: MessageKind::Info,
            text: text.into(),
        }
    }

    pub fn alert<T: Into<String>>(text: T) -> Self {
        Self {
            kind: MessageKind::Alert,
            text: text.into(),
        }
    }
}

#[derive(Debug)]
pub struct DriveReport {
    pub events: Vec<GameEvent>,
    pub awaiting_human: bool,
    pub winner: Option<usize>,
    pub draw: bool,
}

impl DriveReport {
    pub fn game_over(&self) -> bool {
        self.winner.is_some() || self.draw
    }
}

#[derive(Debug)]
pub struct GameState {
    lives: [u8; PLAYERS],
    pub hands: [[Option<CardId>; HAND_SIZE]; PLAYERS],
    pub deck: [CardId; DECK_SIZE],
    pub round_scores: [u8; PLAYERS],
    stack_index: usize,
    rng: StdRng,
    current_player: usize,
    round_start_player: usize,
    stick_player: Option<usize>,
    stop_player: Option<usize>,
    next_start_candidate: usize,
    finished: bool,
    awaiting_human: bool,
    human_old_stack_card: Option<CardId>,
    human_can_draw_next: bool,
    human_can_stick: bool,
}

impl GameState {
    /// Creates a new game state with three lives per player.
    pub fn new(seed: Option<u64>) -> Self {
        let rng = match seed {
            Some(value) => StdRng::seed_from_u64(value),
            None => StdRng::from_entropy(),
        };

        let mut state = Self {
            lives: [3; PLAYERS],
            hands: [[None; HAND_SIZE]; PLAYERS],
            deck: ordered_deck(),
            round_scores: [0; PLAYERS],
            stack_index: 0,
            rng,
            current_player: 0,
            round_start_player: 0,
            stick_player: None,
            stop_player: None,
            next_start_candidate: PLAYERS - 1,
            finished: false,
            awaiting_human: false,
            human_old_stack_card: None,
            human_can_draw_next: false,
            human_can_stick: false,
        };

        state.shuffle_deck();
        state
    }

    /// Resets the game back to its initial state with fresh lives.
    pub fn start_game(&mut self) {
        self.lives = [3; PLAYERS];
        self.round_scores = [0; PLAYERS];
        self.hands = [[None; HAND_SIZE]; PLAYERS];
        self.stack_index = 0;
        self.stick_player = None;
        self.stop_player = None;
        self.current_player = 0;
        self.round_start_player = 0;
        self.next_start_candidate = PLAYERS - 1;
        self.finished = false;
        self.awaiting_human = false;
        self.human_old_stack_card = None;
        self.human_can_draw_next = false;
        self.human_can_stick = false;
        self.shuffle_deck();
    }

    /// Starts a brand-new game and immediately runs until human input is required or the game ends.
    pub fn start_fresh(&mut self) -> DriveReport {
        self.start_game();
        self.start_new_round()
    }

    /// Starts a new round (without resetting lives) and processes automated turns until
    /// human input is required or the game ends.
    pub fn start_new_round(&mut self) -> DriveReport {
        let mut events = Vec::new();
        self.start_round_internal(&mut events);
        self.drive_round_internal(events)
    }

    /// Advances the game after the human has completed their turn.
    pub fn advance_after_human_turn(&mut self) -> DriveReport {
        if self.finished {
            self.awaiting_human = false;
            let alive = self.alive_players();
            return DriveReport {
                events: Vec::new(),
                awaiting_human: false,
                winner: alive.first().copied(),
                draw: alive.is_empty(),
            };
        }

        self.end_human_turn();

        if self.current_player != 0 {
            return self.drive_round_internal(Vec::new());
        }

        if let Some(next) = self.next_alive_after(0) {
            self.current_player = next;
            return self.drive_round_internal(Vec::new());
        }

        let mut events = Vec::new();
        match self.finish_round(&mut events) {
            FinishResult::Continue => self.drive_round_internal(events),
            FinishResult::GameOver { winner, draw } => {
                self.awaiting_human = false;
                DriveReport {
                    events,
                    awaiting_human: false,
                    winner,
                    draw,
                }
            }
        }
    }

    /// Attempts to mark the supplied player as sticking for the remainder of the round.
    pub fn apply_stick(&mut self, player: usize) -> bool {
        if self.finished || self.stick_player.is_some() || self.lives[player] == 0 {
            return false;
        }

        self.stick_player = Some(player);
        true
    }

    /// Human-specific convenience that mirrors the original stick + OK flow.
    pub fn human_stick(&mut self) -> Option<DriveReport> {
        if !self.awaiting_human || !self.human_can_stick {
            return None;
        }

        if !self.apply_stick(0) {
            return None;
        }

        Some(self.advance_after_human_turn())
    }

    pub fn set_lives(&mut self, lives: [u8; PLAYERS]) {
        self.lives = lives;
        self.finished = false;
    }

    pub fn lives(&self) -> &[u8; PLAYERS] {
        &self.lives
    }

    pub fn awaiting_human(&self) -> bool {
        self.awaiting_human
    }

    pub fn human_can_stick(&self) -> bool {
        self.awaiting_human && self.human_can_stick
    }

    pub fn current_player(&self) -> usize {
        self.current_player
    }

    pub fn round_start_player(&self) -> usize {
        self.round_start_player
    }

    pub fn stick_player(&self) -> Option<usize> {
        self.stick_player
    }

    pub fn stack_index(&self) -> usize {
        self.stack_index
    }

    pub fn stack_top_card(&self) -> Option<CardId> {
        self.deck.get(self.stack_index).copied()
    }

    pub fn update_round_scores(&mut self) {
        for player in 0..PLAYERS {
            self.round_scores[player] = if self.lives[player] > 0 {
                hand_max_score(&self.hands[player])
            } else {
                0
            };
        }
    }

    pub fn lowest_alive_score(&self) -> Option<u8> {
        self.round_scores
            .iter()
            .zip(self.lives.iter())
            .filter_map(|(&score, &lives)| if lives > 0 { Some(score) } else { None })
            .min()
    }

    pub fn player_has_stop_the_bus(&self, player: usize) -> bool {
        self.round_scores
            .get(player)
            .map(|&score| score == 31)
            .unwrap_or(false)
    }

    pub fn human_swap_with_stack(&mut self, slot: usize) -> Option<DriveReport> {
        if !self.awaiting_human || self.lives[0] == 0 || slot >= HAND_SIZE {
            return None;
        }

        let stack_card = self.current_stack_card()?;
        let hand_card = self.hands[0][slot]?;

        self.hands[0][slot] = Some(stack_card);
        self.deck[self.stack_index] = hand_card;
        self.update_round_scores();

        let stack_matches_old = self.human_old_stack_card == Some(self.deck[self.stack_index]);
        self.human_can_stick = stack_matches_old && self.stick_player.is_none();
        if !stack_matches_old {
            self.human_can_draw_next = false;
        }

        Some(self.idle_report())
    }

    pub fn human_draw_next_card(&mut self) -> Option<DriveReport> {
        if !self.awaiting_human || !self.human_can_draw_next {
            return None;
        }

        if self.human_old_stack_card != self.current_stack_card() {
            return None;
        }

        if self.stack_index + 1 >= DECK_SIZE {
            return Some(DriveReport {
                events: vec![GameEvent::alert("Deck overflow.".to_string())],
                awaiting_human: true,
                winner: None,
                draw: false,
            });
        }

        self.stack_index += 1;
        self.update_round_scores();
        self.human_can_draw_next = false;
        self.human_can_stick = false;
        Some(self.idle_report())
    }

    fn idle_report(&self) -> DriveReport {
        DriveReport {
            events: Vec::new(),
            awaiting_human: self.awaiting_human,
            winner: None,
            draw: false,
        }
    }

    fn start_round_internal(&mut self, events: &mut Vec<GameEvent>) {
        self.stop_player = None;
        self.stick_player = None;
        self.finished = false;
        self.awaiting_human = false;
        self.human_can_draw_next = false;
        self.human_can_stick = false;
        self.human_old_stack_card = None;
        self.round_scores = [0; PLAYERS];

        self.deal_round();

        if let Some(start) = self.next_alive_after(self.next_start_candidate) {
            self.next_start_candidate = start;
            self.current_player = start;
            self.round_start_player = start;
            events.push(GameEvent::info(format!("Player {} to start", start + 1)));
        } else {
            self.current_player = 0;
            self.round_start_player = 0;
        }
    }

    fn begin_human_turn(&mut self) {
        self.awaiting_human = true;
        self.human_can_draw_next = true;
        self.human_old_stack_card = self.current_stack_card();
        self.human_can_stick = self.stick_player.is_none();
    }

    fn end_human_turn(&mut self) {
        self.awaiting_human = false;
        self.human_can_draw_next = false;
        self.human_can_stick = false;
        self.human_old_stack_card = None;
    }

    fn drive_round_internal(&mut self, mut events: Vec<GameEvent>) -> DriveReport {
        loop {
            self.update_round_scores();

            if self.detect_stop_bus(&mut events) {
                match self.finish_round(&mut events) {
                    FinishResult::Continue => continue,
                    FinishResult::GameOver { winner, draw } => {
                        return DriveReport {
                            events,
                            awaiting_human: false,
                            winner,
                            draw,
                        }
                    }
                }
            }

            if self.finished {
                let alive = self.alive_players();
                let winner = if alive.len() == 1 {
                    Some(alive[0])
                } else {
                    None
                };
                let draw = alive.is_empty();
                return DriveReport {
                    events,
                    awaiting_human: false,
                    winner,
                    draw,
                };
            }

            if self.current_player == 0 {
                if self.lives[0] == 0 {
                    if let Some(next) = self.next_alive_after(0) {
                        self.current_player = next;
                        continue;
                    }

                    return DriveReport {
                        events,
                        awaiting_human: false,
                        winner: None,
                        draw: true,
                    };
                }

                self.begin_human_turn();
                return DriveReport {
                    events,
                    awaiting_human: true,
                    winner: None,
                    draw: false,
                };
            }

            if self.lives[self.current_player] == 0 {
                if let Some(next) = self.next_alive_after(self.current_player) {
                    self.current_player = next;
                    continue;
                }

                match self.finish_round(&mut events) {
                    FinishResult::Continue => continue,
                    FinishResult::GameOver { winner, draw } => {
                        return DriveReport {
                            events,
                            awaiting_human: false,
                            winner,
                            draw,
                        }
                    }
                }
            }

            let active = self.current_player;
            self.execute_auto_turn(active, &mut events);

            if let Some(next) = self.next_alive_after(active) {
                self.current_player = next;
            } else {
                match self.finish_round(&mut events) {
                    FinishResult::Continue => continue,
                    FinishResult::GameOver { winner, draw } => {
                        return DriveReport {
                            events,
                            awaiting_human: false,
                            winner,
                            draw,
                        }
                    }
                }
            }

            if let Some(stick) = self.stick_player {
                if self.current_player == stick {
                    match self.finish_round(&mut events) {
                        FinishResult::Continue => continue,
                        FinishResult::GameOver { winner, draw } => {
                            return DriveReport {
                                events,
                                awaiting_human: false,
                                winner,
                                draw,
                            }
                        }
                    }
                }
            }
        }
    }

    fn detect_stop_bus(&mut self, events: &mut Vec<GameEvent>) -> bool {
        for player in 0..PLAYERS {
            if self.lives[player] == 0 {
                continue;
            }

            if self.player_has_stop_the_bus(player) {
                if self.stop_player != Some(player) {
                    self.stop_player = Some(player);
                    let message = if player == 0 {
                        "You've stopped the bus!".to_string()
                    } else {
                        format!("Player {} has stopped the bus.", player + 1)
                    };
                    events.push(GameEvent::alert(message));
                }
                return true;
            }
        }

        false
    }

    fn finish_round(&mut self, events: &mut Vec<GameEvent>) -> FinishResult {
        self.update_round_scores();

        if let Some(stop_player) = self.stop_player {
            for player in 0..PLAYERS {
                if player != stop_player && self.lives[player] > 0 {
                    self.decrement_life(player, events);
                }
            }
        } else if let Some(lowest) = self.lowest_alive_score() {
            for player in 0..PLAYERS {
                if self.lives[player] > 0 && self.round_scores[player] == lowest {
                    self.decrement_life(player, events);
                }
            }
        }

        self.stop_player = None;
        self.stick_player = None;
        self.awaiting_human = false;
        self.human_can_draw_next = false;
        self.human_can_stick = false;
        self.human_old_stack_card = None;

        let alive = self.alive_players();
        match alive.len() {
            0 => {
                events.push(GameEvent::alert("A draw is declared.".to_string()));
                events.push(GameEvent::info(
                    "That was a rare message - well done!".to_string(),
                ));
                self.finished = true;
                FinishResult::GameOver {
                    winner: None,
                    draw: true,
                }
            }
            1 => {
                let winner = alive[0];
                let message = if winner == 0 {
                    "Congratulations - you've won!".to_string()
                } else {
                    format!("Player {} has won.", winner + 1)
                };
                events.push(GameEvent::info(message));
                self.finished = true;
                FinishResult::GameOver {
                    winner: Some(winner),
                    draw: false,
                }
            }
            _ => {
                self.start_round_internal(events);
                FinishResult::Continue
            }
        }
    }

    fn decrement_life(&mut self, player: usize, events: &mut Vec<GameEvent>) {
        if self.lives[player] == 0 {
            return;
        }

        self.lives[player] = self.lives[player].saturating_sub(1);

        if player == 0 {
            events.push(GameEvent::alert("You've lost a life!".to_string()));
        } else {
            events.push(GameEvent::info(format!(
                "Player {} lost a life.",
                player + 1
            )));
        }

        if self.lives[player] == 0 {
            if player == 0 {
                events.push(GameEvent::alert(
                    "Unfortunately you've been knocked out!".to_string(),
                ));
            } else {
                events.push(GameEvent::info(format!(
                    "Player {} has no lives left.",
                    player + 1
                )));
            }
        }
    }

    fn execute_auto_turn(&mut self, player: usize, events: &mut Vec<GameEvent>) {
        self.update_round_scores();
        let base_score = self.round_scores[player];

        if base_score > 25 && self.stick_player.is_none() {
            self.stick_player = Some(player);
            if player != 0 {
                events.push(GameEvent::info(format!("Player {} sticks.", player + 1)));
            }
            return;
        }

        let stack_card = match self.current_stack_card() {
            Some(card) => card,
            None => return,
        };

        let mut best_slot = None;
        let mut best_score = base_score;

        for slot in 0..HAND_SIZE {
            if let Some(_) = self.hands[player][slot] {
                let mut temp = self.hands[player];
                temp[slot] = Some(stack_card);
                let score = hand_max_score(&temp);
                if score > best_score {
                    best_score = score;
                    best_slot = Some(slot);
                }
            }
        }

        if best_score > base_score && best_score > 6 {
            if let Some(slot) = best_slot {
                self.swap_with_stack(player, slot);
                return;
            }
        }

        if self.stack_index + 1 >= DECK_SIZE {
            events.push(GameEvent::alert("Deck overflow.".to_string()));
            return;
        }

        self.stack_index += 1;

        let stack_card = match self.current_stack_card() {
            Some(card) => card,
            None => return,
        };

        let mut post_best_slot = None;
        let mut post_best_score = base_score;

        for slot in 0..HAND_SIZE {
            if let Some(_) = self.hands[player][slot] {
                let mut temp = self.hands[player];
                temp[slot] = Some(stack_card);
                let score = hand_max_score(&temp);
                if score > post_best_score {
                    post_best_score = score;
                    post_best_slot = Some(slot);
                }
            }
        }

        if post_best_score > base_score {
            if let Some(slot) = post_best_slot {
                self.swap_with_stack(player, slot);
            }
        }
    }

    fn current_stack_card(&self) -> Option<CardId> {
        self.deck.get(self.stack_index).copied()
    }

    fn swap_with_stack(&mut self, player: usize, slot: usize) {
        if self.stack_index >= DECK_SIZE {
            return;
        }

        if let Some(card) = self.hands[player][slot] {
            let stack_card = self.deck[self.stack_index];
            self.hands[player][slot] = Some(stack_card);
            self.deck[self.stack_index] = card;
        }
    }

    fn shuffle_deck(&mut self) {
        self.deck = ordered_deck();

        for _ in 0..100 {
            let a = self.rng.gen_range(0..DECK_SIZE);
            let b = self.rng.gen_range(0..DECK_SIZE);
            self.deck.swap(a, b);
        }

        self.stack_index = 0;
    }

    fn deal_round(&mut self) {
        self.shuffle_deck();

        let mut dealt = 0usize;
        for slot in 0..HAND_SIZE {
            for player in 0..PLAYERS {
                if self.lives[player] == 0 {
                    self.hands[player][slot] = None;
                } else {
                    let card = self.deck[dealt];
                    dealt += 1;
                    self.hands[player][slot] = Some(card);
                }
            }
        }

        self.stack_index = dealt;
        if self.stack_index >= DECK_SIZE {
            self.stack_index = DECK_SIZE - 1;
        }
        self.update_round_scores();
    }

    fn next_alive_after(&self, start: usize) -> Option<usize> {
        for offset in 1..=PLAYERS {
            let idx = (start + offset) % PLAYERS;
            if self.lives[idx] > 0 {
                return Some(idx);
            }
        }
        None
    }

    fn alive_players(&self) -> Vec<usize> {
        (0..PLAYERS).filter(|&p| self.lives[p] > 0).collect()
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new(None)
    }
}

#[derive(Debug)]
enum FinishResult {
    Continue,
    GameOver { winner: Option<usize>, draw: bool },
}

fn ordered_deck() -> [CardId; DECK_SIZE] {
    let mut deck = [0u8; DECK_SIZE];
    for (index, card) in deck.iter_mut().enumerate() {
        *card = (index + 1) as CardId;
    }
    deck
}

pub fn card_rank(card: CardId) -> Option<u8> {
    if !(1..=52).contains(&card) {
        return None;
    }
    let rank = ((card - 1) % 13) + 1;
    Some(rank)
}

pub fn card_suit(card: CardId) -> Option<Suit> {
    if !(1..=52).contains(&card) {
        return None;
    }
    let bucket = (card - 1) / 13;
    let suit = match bucket {
        0 => Suit::Clubs,
        1 => Suit::Diamonds,
        2 => Suit::Hearts,
        _ => Suit::Spades,
    };
    Some(suit)
}

pub fn card_points(card: CardId) -> u8 {
    match card_rank(card).unwrap_or(0) {
        1 => 11,
        11 | 12 | 13 => 10,
        value => value,
    }
}

pub fn hand_max_score(cards: &[Option<CardId>; HAND_SIZE]) -> u8 {
    let mut max_score = 0u8;

    for (index, candidate) in cards.iter().enumerate() {
        let Some(card) = candidate else {
            continue;
        };

        let Some(suit) = card_suit(*card) else {
            continue;
        };

        let mut score = card_points(*card);
        for offset in 1..HAND_SIZE {
            let idx = (index + offset) % HAND_SIZE;
            if let Some(other) = cards[idx] {
                if card_suit(other) == Some(suit) {
                    score += card_points(other);
                }
            }
        }

        max_score = max_score.max(score);
    }

    max_score
}

pub fn player_stop_the_bus(cards: &[Option<CardId>; HAND_SIZE]) -> bool {
    hand_max_score(cards) == 31
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_and_suits() {
        assert_eq!(card_rank(1), Some(1));
        assert_eq!(card_rank(13), Some(13));
        assert_eq!(card_rank(14), Some(1));
        assert_eq!(card_suit(1), Some(Suit::Clubs));
        assert_eq!(card_suit(14), Some(Suit::Diamonds));
        assert_eq!(card_suit(27), Some(Suit::Hearts));
        assert_eq!(card_suit(40), Some(Suit::Spades));
    }

    #[test]
    fn value_scoring_matches_pascal_logic() {
        assert_eq!(card_points(1), 11);
        assert_eq!(card_points(13), 10);
        assert_eq!(card_points(12), 10);
        assert_eq!(card_points(11), 10);
        assert_eq!(card_points(10), 10);
        assert_eq!(card_points(8), 8);
    }

    #[test]
    fn hand_scoring_respects_best_suit() {
        let hand = [Some(1), Some(14), Some(27)];
        assert_eq!(hand_max_score(&hand), 11);

        let same_suit = [Some(1), Some(2), Some(3)];
        assert_eq!(hand_max_score(&same_suit), 16);

        let mixed = [Some(1), Some(14), Some(2)];
        assert_eq!(hand_max_score(&mixed), 13);
    }

    #[test]
    fn detect_stop_the_bus() {
        let hand = [Some(27), Some(36), Some(39)];
        assert!(player_stop_the_bus(&hand));

        let non_stop = [Some(1), Some(14), Some(28)];
        assert!(!player_stop_the_bus(&non_stop));
    }

    #[test]
    fn deal_respects_lives() {
        let mut game = GameState::new(Some(42));
        game.set_lives([3, 0, 1, 0]);
        game.deal_round();

        assert!(game.hands[0].iter().all(|c| c.is_some()));
        assert!(game.hands[1].iter().all(|c| c.is_none()));
        assert!(game.hands[2].iter().all(|c| c.is_some()));
        assert!(game.hands[3].iter().all(|c| c.is_none()));

        assert_eq!(game.stack_index(), 6);
        assert!(game.lowest_alive_score().is_some());
    }
}
