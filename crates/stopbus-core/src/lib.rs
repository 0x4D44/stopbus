use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub type CardId = u8;
pub const DECK_SIZE: usize = 52;
pub const PLAYERS: usize = 4;

struct LifeLossInfo {
    player: usize,
    knocked_out: bool,
}

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
    pub turn_sequence: Vec<usize>,
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
    round_turns: u16,
    stick_player: Option<usize>,
    stick_player_score: Option<u8>,
    pending_new_round: bool,
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
            round_turns: 0,
            stick_player: None,
            stick_player_score: None,
            pending_new_round: false,
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
        self.stick_player_score = None;
        self.pending_new_round = false;
        self.stop_player = None;
        self.current_player = 0;
        self.round_start_player = 0;
        self.round_turns = 0;
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
        self.drive_round_step(events)
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
                turn_sequence: Vec::new(),
            };
        }

        self.end_human_turn();

        if let Some(next) = self.next_alive_after(0) {
            self.current_player = next;
            return self.drive_round_step(Vec::new());
        }

        // Everyone dead
        let mut events = Vec::new();
        if let FinishResult::GameOver { winner, draw } = self.finish_round(&mut events) {
            return DriveReport {
                events,
                awaiting_human: false,
                winner,
                draw,
                turn_sequence: Vec::new(),
            };
        }
        unreachable!();
    }

    pub fn continue_automation(&mut self) -> DriveReport {
        self.drive_round_step(Vec::new())
    }

    /// Attempts to mark the supplied player as sticking for the remainder of the round.
    pub fn apply_stick(&mut self, player: usize) -> bool {
        if self.finished || self.stick_player.is_some() || self.lives[player] == 0 {
            return false;
        }

        self.update_round_scores();
        let score = self.round_scores[player];
        self.stick_player = Some(player);
        self.stick_player_score = Some(score);
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
        if !stack_matches_old {
            self.human_can_draw_next = false;
        }
        self.refresh_human_stick_flag();

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
                turn_sequence: Vec::new(),
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
            turn_sequence: Vec::new(),
        }
    }

    fn start_round_internal(&mut self, _events: &mut [GameEvent]) {
        self.stop_player = None;
        self.stick_player = None;
        self.stick_player_score = None;
        self.finished = false;
        self.awaiting_human = false;
        self.human_can_draw_next = false;
        self.human_can_stick = false;
        self.human_old_stack_card = None;
        self.pending_new_round = false;
        self.round_scores = [0; PLAYERS];
        self.round_turns = 0;

        self.deal_round();

        if let Some(start) = self.next_alive_after(self.next_start_candidate) {
            self.next_start_candidate = start;
            self.current_player = start;
            self.round_start_player = start;
        } else {
            self.current_player = 0;
            self.round_start_player = 0;
        }
    }

    fn begin_human_turn(&mut self) {
        self.awaiting_human = true;
        self.human_can_draw_next = true;
        self.human_old_stack_card = self.current_stack_card();
        self.refresh_human_stick_flag();
    }

    fn end_human_turn(&mut self) {
        self.awaiting_human = false;
        self.human_can_draw_next = false;
        self.human_can_stick = false;
        self.human_old_stack_card = None;
        self.complete_turn();
    }

    fn drive_round_step(&mut self, mut events: Vec<GameEvent>) -> DriveReport {
        if self.pending_new_round {
            self.pending_new_round = false;
            self.start_round_internal(&mut events);
        }

        loop {
            self.update_round_scores();

            if self.detect_stop_bus(&mut events) {
                match self.finish_round(&mut events) {
                    FinishResult::Continue => {
                        return DriveReport {
                            events,
                            awaiting_human: false,
                            winner: None,
                            draw: false,
                            turn_sequence: Vec::new(),
                        };
                    }
                    FinishResult::GameOver { winner, draw } => {
                        return DriveReport {
                            events,
                            awaiting_human: false,
                            winner,
                            draw,
                            turn_sequence: Vec::new(),
                        };
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
                    turn_sequence: Vec::new(),
                };
            }

            if let Some(stick) = self.stick_player {
                if self.current_player == stick {
                    match self.finish_round(&mut events) {
                        FinishResult::Continue => {
                            return DriveReport {
                                events,
                                awaiting_human: false,
                                winner: None,
                                draw: false,
                                turn_sequence: Vec::new(),
                            };
                        }
                        FinishResult::GameOver { winner, draw } => {
                            return DriveReport {
                                events,
                                awaiting_human: false,
                                winner,
                                draw,
                                turn_sequence: Vec::new(),
                            };
                        }
                    }
                }
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
                        turn_sequence: Vec::new(),
                    };
                }

                self.begin_human_turn();
                return DriveReport {
                    events,
                    awaiting_human: true,
                    winner: None,
                    draw: false,
                    turn_sequence: Vec::new(),
                };
            }

            if self.lives[self.current_player] == 0 {
                if let Some(next) = self.next_alive_after(self.current_player) {
                    self.current_player = next;
                    continue;
                }

                // Everyone dead
                if let FinishResult::GameOver { winner, draw } = self.finish_round(&mut events) {
                    return DriveReport {
                        events,
                        awaiting_human: false,
                        winner,
                        draw,
                        turn_sequence: Vec::new(),
                    };
                }
                unreachable!();
            }

            let active = self.current_player;
            self.execute_auto_turn(active, &mut events);

            if let Some(next) = self.next_alive_after(active) {
                self.current_player = next;
            } else {
                // Everyone dead
                if let FinishResult::GameOver { winner, draw } = self.finish_round(&mut events) {
                    return DriveReport {
                        events,
                        awaiting_human: false,
                        winner,
                        draw,
                        turn_sequence: Vec::new(),
                    };
                }
                unreachable!();
            }

            return DriveReport {
                events,
                awaiting_human: false,
                winner: None,
                draw: false,
                turn_sequence: vec![active],
            };
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

        let mut life_losses: Vec<LifeLossInfo> = Vec::new();
        let stop_player = self.stop_player;

        if let Some(stopper) = stop_player {
            for player in 0..PLAYERS {
                if player != stopper && self.lives[player] > 0 {
                    self.decrement_life(player, &mut life_losses);
                }
            }
        } else if let Some(lowest) = self.lowest_alive_score() {
            for player in 0..PLAYERS {
                if self.lives[player] > 0 && self.round_scores[player] == lowest {
                    self.decrement_life(player, &mut life_losses);
                }
            }
        }

        self.stop_player = None;
        self.stick_player = None;
        self.stick_player_score = None;
        self.awaiting_human = false;
        self.human_can_draw_next = false;
        self.human_can_stick = false;
        self.round_turns = 0;
        self.human_old_stack_card = None;

        let loss_players: Vec<usize> = life_losses.iter().map(|info| info.player).collect();
        let knockout_players: Vec<usize> = life_losses
            .iter()
            .filter(|info| info.knocked_out)
            .map(|info| info.player)
            .collect();
        let human_lost = loss_players.contains(&0);
        let human_knocked_out = knockout_players.contains(&0);

        let mut summary_sentences = Vec::new();
        let alert_needed = stop_player == Some(0) || human_lost || human_knocked_out;

        if let Some(stopper) = stop_player {
            let stopper_sentence = if stopper == 0 {
                "You stopped the bus.".to_string()
            } else {
                format!("{} stopped the bus.", self.player_name(stopper))
            };
            summary_sentences.push(stopper_sentence);
        }

        if !loss_players.is_empty() {
            summary_sentences.push(self.loss_sentence(&loss_players));
        }

        for player in knockout_players {
            summary_sentences.push(self.knockout_sentence(player));
        }

        let alive = self.alive_players();

        match alive.len() {
            0 => {
                summary_sentences.push("A draw is declared.".to_string());
                summary_sentences.push("That was a rare message - well done!".to_string());
                self.finished = true;
                let text = summary_sentences.join("\n");
                events.push(GameEvent::alert(text));
                FinishResult::GameOver {
                    winner: None,
                    draw: true,
                }
            }
            1 => {
                let winner = alive[0];
                let winner_sentence = if winner == 0 {
                    "Congratulations - you've won!".to_string()
                } else {
                    format!("{} has won.", self.player_name(winner))
                };
                summary_sentences.push(winner_sentence);
                self.finished = true;
                let text = summary_sentences.join("\n");
                let event = if alert_needed {
                    GameEvent::alert(text)
                } else {
                    GameEvent::info(text)
                };
                events.push(event);
                FinishResult::GameOver {
                    winner: Some(winner),
                    draw: false,
                }
            }
            _ => {
                if !summary_sentences.is_empty() {
                    let text = summary_sentences.join("\n");
                    let event = if alert_needed {
                        GameEvent::alert(text)
                    } else {
                        GameEvent::info(text)
                    };
                    events.push(event);
                }
                self.pending_new_round = true;
                FinishResult::Continue
            }
        }
    }

    fn player_name(&self, player: usize) -> String {
        if player == 0 {
            "You".to_string()
        } else {
            format!("Player {}", player + 1)
        }
    }

    fn join_name_list(names: &[String]) -> String {
        match names.len() {
            0 => String::new(),
            1 => names[0].clone(),
            2 => format!("{} and {}", names[0], names[1]),
            _ => {
                let mut result = names[..names.len() - 1].join(", ");
                result.push_str(", and ");
                result.push_str(&names[names.len() - 1]);
                result
            }
        }
    }

    fn loss_sentence(&self, players: &[usize]) -> String {
        let names: Vec<String> = players.iter().map(|&p| self.player_name(p)).collect();

        if names.is_empty() {
            return String::new();
        }

        if players.len() == 1 && players[0] == 0 {
            "You lost a life.".to_string()
        } else {
            format!("{} lost a life.", Self::join_name_list(&names))
        }
    }

    fn knockout_sentence(&self, player: usize) -> String {
        if player == 0 {
            "You have been knocked out.".to_string()
        } else {
            format!("{} has been knocked out.", self.player_name(player))
        }
    }

    fn decrement_life(&mut self, player: usize, losses: &mut Vec<LifeLossInfo>) {
        if self.lives[player] == 0 {
            return;
        }

        self.lives[player] = self.lives[player].saturating_sub(1);

        let knocked_out = self.lives[player] == 0;
        losses.push(LifeLossInfo {
            player,
            knocked_out,
        });
    }

    fn execute_auto_turn(&mut self, player: usize, events: &mut Vec<GameEvent>) {
        self.update_round_scores();
        let base_score = self.round_scores[player];

        if self.stick_player.is_none() && base_score > 25 {
            self.mark_player_sticking(player, events);
            self.complete_turn();
            return;
        }

        if self.try_swap_for_improvement(player, base_score, true) {
            self.complete_turn();
            return;
        }

        if !self.advance_stack_pointer(events) {
            self.complete_turn();
            return;
        }

        self.update_round_scores();
        let second_base = self.round_scores[player];
        let _ = self.try_swap_for_improvement(player, second_base, false);

        self.complete_turn();
    }

    fn mark_player_sticking(&mut self, player: usize, _events: &mut [GameEvent]) {
        self.update_round_scores();
        let score = self.round_scores[player];
        if !self.can_player_stick(player, score) {
            return;
        }

        self.stick_player = Some(player);
        self.stick_player_score = Some(score);
    }

    fn try_swap_for_improvement(
        &mut self,
        player: usize,
        base_score: u8,
        require_gt_six: bool,
    ) -> bool {
        let Some(stack_card) = self.current_stack_card() else {
            return false;
        };

        let mut best_slot = None;
        let mut best_score = base_score;

        for slot in 0..HAND_SIZE {
            if self.hands[player][slot].is_some() {
                let mut temp = self.hands[player];
                temp[slot] = Some(stack_card);
                let score = hand_max_score(&temp);
                if score > best_score {
                    best_score = score;
                    best_slot = Some(slot);
                }
            }
        }

        let improvement = if require_gt_six {
            best_score > base_score && best_score > 6
        } else {
            best_score > base_score
        };

        if improvement {
            if let Some(slot) = best_slot {
                self.swap_with_stack(player, slot);
                self.update_round_scores();
                return true;
            }
        }

        false
    }

    fn advance_stack_pointer(&mut self, events: &mut Vec<GameEvent>) -> bool {
        if self.stack_index + 1 >= DECK_SIZE {
            events.push(GameEvent::alert("Deck overflow.".to_string()));
            return false;
        }

        self.stack_index += 1;
        true
    }

    fn can_player_stick(&self, player: usize, _score: u8) -> bool {
        if self.finished || self.lives[player] == 0 {
            return false;
        }

        match self.stick_player {
            Some(current) => current == player,
            None => true,
        }
    }

    fn complete_turn(&mut self) {
        self.round_turns = self.round_turns.saturating_add(1);
    }

    fn refresh_human_stick_flag(&mut self) {
        if !self.awaiting_human || self.lives[0] == 0 {
            self.human_can_stick = false;
            return;
        }

        self.update_round_scores();
        let stack_matches_old = self.human_old_stack_card == self.current_stack_card();
        let allowed = stack_matches_old && self.stick_player.is_none();
        self.human_can_stick = allowed;
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
        11..=13 => 10,
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

    #[test]
    fn test_start_game_resets_lives_and_scores() {
        let mut game = GameState::new(None);
        game.set_lives([1, 1, 1, 1]);
        game.round_scores = [30, 30, 30, 30];

        game.start_game();

        assert_eq!(game.lives, [3, 3, 3, 3]);
        assert_eq!(game.round_scores, [0, 0, 0, 0]);
        assert_eq!(game.stack_index, 0);
        assert!(!game.finished);
    }

    #[test]
    fn test_start_new_round_preserves_lives() {
        let mut game = GameState::new(None);
        game.set_lives([2, 2, 2, 2]);
        game.start_new_round();

        assert_eq!(game.lives, [2, 2, 2, 2]);
        assert!(game.hands[0][0].is_some());
    }

    #[test]
    fn test_apply_stick_valid_and_invalid() {
        let mut game = GameState::new(None);
        game.start_game();
        game.deal_round();

        assert!(game.apply_stick(0));
        assert_eq!(game.stick_player, Some(0));
        assert!(game.stick_player_score.is_some());

        assert!(!game.apply_stick(1));
        assert_eq!(game.stick_player, Some(0));
    }

    #[test]
    fn test_human_stick_flow() {
        let mut game = GameState::new(None);
        game.start_game();

        game.awaiting_human = true;
        game.human_can_stick = true;
        game.lives[0] = 3;

        let result = game.human_stick();
        assert!(result.is_some());
        assert_eq!(game.stick_player, Some(0));
        assert!(!game.awaiting_human);
    }

    #[test]
    fn test_human_swap_with_stack_valid() {
        let mut game = GameState::new(Some(123));
        game.start_new_round();

        game.awaiting_human = true;
        game.lives[0] = 3;

        let old_hand_card = game.hands[0][0];
        let old_stack_card = game.stack_top_card();
        let old_stack_idx = game.stack_index;

        let report = game.human_swap_with_stack(0);
        assert!(report.is_some());

        assert_eq!(game.hands[0][0], old_stack_card);
        assert_eq!(game.stack_top_card(), old_hand_card);
        assert_eq!(game.stack_index, old_stack_idx);

        assert!(!game.human_can_stick);
    }

    #[test]
    fn test_human_swap_invalid_indices() {
        let mut game = GameState::new(None);
        game.start_new_round();
        game.awaiting_human = true;
        game.lives[0] = 3;

        assert!(game.human_swap_with_stack(3).is_none());
    }

    #[test]
    fn test_human_draw_next_card() {
        let mut game = GameState::new(None);
        game.start_new_round();

        game.awaiting_human = true;
        game.human_can_draw_next = true;
        game.human_old_stack_card = game.current_stack_card();

        let old_index = game.stack_index;
        let report = game.human_draw_next_card();

        assert!(report.is_some());
        assert_eq!(game.stack_index, old_index + 1);
        assert!(!game.human_can_draw_next);
    }

    #[test]
    fn test_human_cannot_act_when_not_awaiting() {
        let mut game = GameState::new(None);
        game.start_new_round();
        game.awaiting_human = false; // explicitly false

        assert!(game.human_stick().is_none());
        assert!(game.human_swap_with_stack(0).is_none());
        assert!(game.human_draw_next_card().is_none());
    }

    #[test]
    fn test_ai_sticks_on_high_score() {
        let mut game = GameState::new(None);
        game.start_new_round();

        // King, Queen, Jack of Clubs = 30 points
        game.hands[1] = [Some(13), Some(12), Some(11)];
        // Ensure others don't have Stop the Bus
        game.hands[0] = [Some(2), Some(3), Some(4)];
        game.hands[2] = [Some(5), Some(6), Some(7)];
        game.hands[3] = [Some(8), Some(9), Some(10)];
        game.update_round_scores();

        game.lives = [3, 3, 3, 3];
        game.current_player = 1;
        game.awaiting_human = false;

        // Run until human turn
        let _ = game.continue_automation();
        while !game.awaiting_human && !game.finished {
            let _ = game.continue_automation();
        }

        assert_eq!(game.stick_player, Some(1));
        assert_eq!(game.stick_player_score, Some(30));

        assert!(game.awaiting_human);
        assert_eq!(game.current_player, 0);
    }

    #[test]
    fn test_ai_swaps_for_better_card() {
        let mut game = GameState::new(None);
        game.start_new_round();

        // P1: 2 Clubs(2), 3 Clubs(3), 2 Hearts(15). Score: 5 (Clubs).
        game.hands[1] = [Some(2), Some(3), Some(15)];

        // Stack: Ace Clubs (1).
        let stack_idx = game.stack_index;
        game.deck[stack_idx] = 1;

        game.current_player = 1;
        let _ = game.continue_automation();

        // P1 should have swapped for Ace Clubs (1).
        assert!(game.hands[1].contains(&Some(1)));
        // Stack should no longer have Ace Clubs
        assert_ne!(game.deck[stack_idx], 1);
    }

    #[test]
    fn test_ai_draws_when_stack_bad() {
        let mut game = GameState::new(None);
        game.start_new_round();

        // P1: Ace, King, Queen of Clubs (30 pts)
        // Hand is good, but let's force a situation where swap is bad but stick isn't triggered yet?
        // Wait, 30 triggers stick (>25).
        // Let's give mediocre hand: 8, 9, 10 Clubs (27) -> Sticks.
        // Mediocre low: 2, 3, 4 Clubs (9 pts).
        game.hands[1] = [Some(2), Some(3), Some(4)];

        // Stack: 2 Hearts (15). (Score if taken: 2,3,4=9. 2H doesn't help much).
        // Actually 2H is isolated. 9 is better than 4? No.
        let stack_idx = game.stack_index;
        game.deck[stack_idx] = 15;

        game.current_player = 1;
        let _ = game.continue_automation();

        // P1 should NOT have taken 15.
        assert!(!game.hands[1].contains(&Some(15)));
        // Should have advanced stack (draw)
        assert_eq!(game.stack_index, stack_idx + 1);
    }

    #[test]
    fn test_finish_round_lowest_score_loses_life() {
        let mut game = GameState::new(None);
        game.start_new_round();
        game.lives = [3, 3, 3, 3];

        // P0 Sticks.
        game.stick_player = Some(0);

        // P0=25: K, Q, 5 (Clubs)
        game.hands[0] = [Some(13), Some(12), Some(5)];
        // P1=30: K, Q, J (Diamonds) -> 26, 25, 24
        game.hands[1] = [Some(26), Some(25), Some(24)];
        // P2=10: 10 (Hearts), 2 (Spades), 3 (Spades). Score 10 (Hearts) or 5 (Spades).
        // Let's use single high card 10. 10 of Spades is 49.
        game.hands[2] = [Some(49), Some(2), Some(3)]; // 10S (10), 2C(2), 3C(3). Max 10.
                                                      // P3=10: Same logic.
        game.hands[3] = [Some(49), Some(2), Some(3)];

        game.update_round_scores();
        game.current_player = 0;

        let report = game.continue_automation();

        // P2 and P3 lost life
        assert_eq!(game.lives, [3, 3, 2, 2]);
        // P0 and P1 safe
        assert_eq!(game.lives[0], 3);
        assert_eq!(game.lives[1], 3);

        let text = report
            .events
            .iter()
            .map(|e| &e.text)
            .fold(String::new(), |a, b| a + b);
        assert!(text.contains("Player 3 and Player 4 lost a life"));
    }

    #[test]
    fn test_stop_the_bus_protection() {
        let mut game = GameState::new(None);
        game.start_new_round();
        game.lives = [3, 3, 3, 3];

        // P0 gets 31.
        game.hands[0] = [Some(1), Some(13), Some(12)]; // Ace, K, Q Clubs
                                                       // Others have low scores
        game.hands[1] = [Some(2), Some(3), Some(4)];
        game.hands[2] = [Some(2), Some(3), Some(4)];
        game.hands[3] = [Some(2), Some(3), Some(4)];

        game.update_round_scores();
        game.current_player = 0;

        let report = game.continue_automation();

        // Everyone else should lose a life
        assert_eq!(game.lives, [3, 2, 2, 2]);
        assert!(report
            .events
            .iter()
            .any(|e| e.text.contains("You stopped the bus")));
    }

    #[test]
    fn test_game_over_winner_declared() {
        let mut game = GameState::new(None);
        game.start_new_round();
        game.lives = [1, 1, 0, 0];

        // P0 Sticks.
        game.stick_player = Some(0);

        // P0=30
        game.hands[0] = [Some(13), Some(12), Some(11)];
        // P1=10
        game.hands[1] = [Some(2), Some(3), Some(4)];

        game.update_round_scores();
        game.current_player = 0;

        let report = game.continue_automation();

        // P1 dies. P0 wins.
        assert_eq!(game.lives, [1, 0, 0, 0]);
        assert!(game.finished);
        assert_eq!(report.winner, Some(0));
    }

    #[test]
    fn test_drive_handles_immediate_stop_bus() {
        let mut game = GameState::new(None);
        game.start_new_round();

        // Set P1 to have 31 immediately
        game.hands[1] = [Some(1), Some(13), Some(12)]; // 31
        game.update_round_scores();

        // Enable human stick
        game.awaiting_human = true;
        game.human_can_stick = true;

        let report = game.human_stick().expect("Human stick failed");

        // Events should show P1 stopped bus
        assert!(report
            .events
            .iter()
            .any(|e| e.text.contains("Player 2 has stopped the bus")));
    }

    #[test]
    fn test_getters_and_simple_methods() {
        let mut game = GameState::new(None);
        game.set_lives([3, 2, 1, 0]);
        assert_eq!(game.lives(), &[3, 2, 1, 0]);
        assert!(!game.awaiting_human());
        assert!(!game.human_can_stick());
        assert_eq!(game.current_player(), 0);
        assert_eq!(game.round_start_player(), 0);
        assert_eq!(game.stick_player(), None);
        assert!(!DriveReport {
            events: vec![],
            awaiting_human: false,
            winner: None,
            draw: false,
            turn_sequence: vec![]
        }
        .game_over());
        assert!(DriveReport {
            events: vec![],
            awaiting_human: false,
            winner: Some(0),
            draw: false,
            turn_sequence: vec![]
        }
        .game_over());
        assert!(DriveReport {
            events: vec![],
            awaiting_human: false,
            winner: None,
            draw: true,
            turn_sequence: vec![]
        }
        .game_over());

        let game2 = GameState::default();
        assert_eq!(game2.lives(), &[3, 3, 3, 3]);
    }

    #[test]
    fn test_human_draw_deck_overflow() {
        let mut game = GameState::new(None);
        game.start_new_round();
        game.stack_index = DECK_SIZE - 1;
        game.awaiting_human = true;
        game.human_can_draw_next = true;
        game.human_old_stack_card = game.current_stack_card();

        let report = game.human_draw_next_card().unwrap();
        assert!(report.events[0].text.contains("Deck overflow"));
        assert!(report.awaiting_human);
    }

    #[test]
    fn test_advance_stack_overflow() {
        let mut game = GameState::new(None);
        game.stack_index = DECK_SIZE - 1;
        let mut events = Vec::new();
        assert!(!game.advance_stack_pointer(&mut events));
        assert!(events[0].text.contains("Deck overflow"));
    }

    #[test]
    fn test_draw_scenario() {
        let mut game = GameState::new(None);
        game.start_new_round();
        // Force everyone to 0 lives except P0, P1
        game.lives = [1, 1, 0, 0];
        // Both have lowest score
        game.hands[0] = [Some(2), Some(3), Some(4)];
        game.hands[1] = [Some(2), Some(3), Some(4)];
        game.update_round_scores();
        game.stick_player = Some(0);
        game.current_player = 0;

        let report = game.continue_automation();
        assert!(report.draw);
        assert!(report
            .events
            .iter()
            .any(|e| e.text.contains("draw is declared")));
    }

    #[test]
    fn test_card_helpers_invalid() {
        assert!(card_rank(0).is_none());
        assert!(card_rank(53).is_none());
        assert!(card_suit(0).is_none());
        assert!(card_suit(53).is_none());
    }

    #[test]
    fn test_human_can_stick_edge_cases() {
        let mut game = GameState::new(None);
        game.awaiting_human = true;
        game.lives[0] = 0;
        game.refresh_human_stick_flag();
        assert!(!game.human_can_stick());

        game.lives[0] = 3;
        game.awaiting_human = true;
        game.stick_player = Some(1);
        game.refresh_human_stick_flag();
        assert!(!game.human_can_stick());
    }

    #[test]
    fn test_start_fresh() {
        let mut game = GameState::new(None);
        let _ = game.start_fresh();
        assert_eq!(game.lives(), &[3, 3, 3, 3]);
    }

    #[test]
    fn test_advance_after_human_turn_finished() {
        let mut game = GameState::new(None);
        game.finished = true;
        let report = game.advance_after_human_turn();
        assert!(!report.awaiting_human);
    }

    #[test]
    fn test_loop_scenarios_and_error_paths() {
        let mut game = GameState::new(None);

        // Scenario: Player 0 (Human) is dead. Round should skip to Player 1.
        game.start_new_round();
        game.lives[0] = 0;
        game.lives[1] = 3;
        game.current_player = 0;
        let report = game.continue_automation();
        assert_eq!(report.turn_sequence, vec![1]);
        assert_eq!(game.current_player, 2);

        // Scenario: Everyone dead. Draw.
        game.lives = [0, 0, 0, 0];
        game.finished = true;
        let report = game.continue_automation();
        assert!(report.draw);

        // Scenario: Pending new round.
        game.lives = [3, 3, 3, 3];
        game.pending_new_round = true;
        game.next_start_candidate = 3; // Forces P0 to start (next alive after 3 is 0)
        let _ = game.continue_automation();
        assert!(game.awaiting_human);
        // Scenario: AI player is dead. Should skip to next alive.
        game.lives = [3, 0, 3, 3];
        game.current_player = 1;
        game.awaiting_human = false;
        game.finished = false;
        game.pending_new_round = false; // explicitly false to not restart
        let report = game.continue_automation();
        assert_eq!(report.turn_sequence, vec![2]);
        assert_eq!(game.current_player, 3);

        // Scenario: Only winner remains
        game.lives = [3, 0, 0, 0];
        game.finished = true;
        let report = game.continue_automation();
        assert!(report.game_over());
        assert_eq!(report.winner, Some(0));

        // Coverage for finish_round branches (alert needed etc)
        game.lives = [1, 1, 1, 1];
        game.hands[0] = [Some(2), Some(3), Some(4)]; // Low score
        game.hands[1] = [Some(1), Some(13), Some(12)]; // Stop the bus
        game.update_round_scores();
        game.current_player = 1;
        let report = game.continue_automation();
        assert!(report.events.iter().any(|e| e.kind == MessageKind::Alert));
    }

    #[test]
    fn test_human_stick_failure() {
        let mut game = GameState::new(None);
        game.start_new_round();
        game.awaiting_human = true;
        game.human_can_stick = true;
        game.lives[0] = 0; // Force apply_stick to fail
        assert!(game.human_stick().is_none());
    }

    #[test]
    fn test_human_draw_mismatch() {
        let mut game = GameState::new(None);
        game.start_new_round();
        game.awaiting_human = true;
        game.human_can_draw_next = true;
        game.human_old_stack_card = Some(99); // Mismatch
        assert!(game.human_draw_next_card().is_none());
    }

    #[test]
    fn test_debug_scores_and_logic() {
        // Verify math
        let h = [Some(2), Some(3), Some(4)]; // 9
        assert_eq!(hand_max_score(&h), 9, "Base hand score wrong");
        let h2 = [Some(15), Some(3), Some(4)]; // 7
        assert_eq!(hand_max_score(&h2), 7, "Swap hand score wrong");

        // Verify stack index logic
        let mut game = GameState::new(Some(1));
        game.stack_index = 51;
        let mut events = Vec::new();
        assert!(!game.advance_stack_pointer(&mut events));
        assert!(!events.is_empty());
    }

    #[test]
    fn test_finish_round_game_over_inside_advance() {
        let mut game = GameState::new(Some(1));
        game.start_new_round();
        game.lives = [3, 1, 0, 0];
        game.stop_player = None;

        game.hands[0] = [Some(13), Some(12), Some(11)]; // 30
        game.hands[1] = [Some(2), Some(3), Some(4)]; // 9
        game.update_round_scores();

        // Simulate P0 sticking and P1 having played, returning to P0
        game.stick_player = Some(0);
        game.stick_player_score = Some(30);
        game.current_player = 0;

        let report = game.continue_automation();

        assert_eq!(
            game.lives[1], 0,
            "P1 should be dead. Events: {:?}",
            report.events
        );
        assert!(report.game_over());
    }

    #[test]
    fn test_start_round_everyone_dead() {
        let mut game = GameState::new(None);
        game.lives = [0; 4];
        let _ = game.start_new_round();
        assert_eq!(game.current_player, 0);
    }

    #[test]
    fn test_drive_everyone_dead() {
        let mut game = GameState::new(None);
        game.lives = [0; 4];
        game.current_player = 0;
        let report = game.continue_automation();
        assert!(report.draw);
    }

    #[test]
    fn test_ai_deck_overflow() {
        let mut game = GameState::new(Some(1));
        game.start_new_round();
        game.lives = [3, 3, 3, 3];
        game.current_player = 1;
        game.hands[1] = [Some(2), Some(3), Some(4)]; // 9 pts
        game.update_round_scores();
        game.stack_index = 100; // Force overflow logic

        // Stick player set to ensure P1 doesn't stick
        game.stick_player = Some(0);

        let report = game.continue_automation();
        assert!(
            report
                .events
                .iter()
                .any(|e| e.text.contains("Deck overflow")),
            "Events: {:?}",
            report.events
        );
    }

    #[test]
    fn test_detect_stop_bus_redundant() {
        let mut game = GameState::new(None);
        game.start_new_round();
        game.lives = [3, 3, 3, 3];
        game.stop_player = Some(0); // Already set
        game.hands[0] = [Some(1), Some(13), Some(12)]; // 31
        game.update_round_scores();

        let mut events = Vec::new();
        assert!(game.detect_stop_bus(&mut events));
        // Should NOT have added another alert
        assert!(events.is_empty());
    }

    #[test]
    fn test_advance_finish_round_branches() {
        // Scenario: GameOver inside advance
        let mut game = GameState::new(None);
        game.lives = [3, 0, 0, 0];
        game.awaiting_human = true;
        game.human_can_stick = true;
        let report = game.human_stick().unwrap();
        assert!(report.game_over());

        // Scenario: Continue inside advance
        let mut game = GameState::new(None);
        game.lives = [3, 3, 3, 3];
        game.hands[0] = [Some(1), Some(13), Some(12)]; // 31
        game.update_round_scores();
        game.awaiting_human = true;
        let report = game.advance_after_human_turn();
        assert!(!report.game_over());
        assert!(game.pending_new_round);
    }

    #[test]
    fn test_extra_loop_branches() {
        let mut game = GameState::new(None);

        // Coverage for drive_round_step: AI dead, no next alive
        game.lives = [0, 0, 0, 0];
        game.current_player = 1;
        game.finished = false;
        let report = game.continue_automation();
        assert!(report.draw);

        // Coverage for drive_round_step: AI played, lone survivor
        let mut game = GameState::new(None);
        game.lives = [0, 3, 0, 0];
        game.current_player = 1;
        game.awaiting_human = false;
        game.finished = false;
        let report = game.continue_automation();
        // Returns report for the turn taken
        assert_eq!(report.turn_sequence, vec![1]);
    }

    #[test]
    fn test_ai_wins_human_safe() {
        let mut game = GameState::new(None);
        game.start_new_round();
        // P1 High, P0 Low
        game.lives = [1, 1, 0, 0];
        game.hands[1] = [Some(1), Some(13), Some(12)]; // 31 (Stops bus)
        game.hands[0] = [Some(2), Some(3), Some(4)]; // 9
        game.update_round_scores();

        game.current_player = 1;
        let report = game.continue_automation();

        // P1 stopped bus. P0 lost life -> 0.
        assert!(report.game_over());
        assert_eq!(report.winner, Some(1));
        assert!(report
            .events
            .iter()
            .any(|e| e.text.contains("Player 2 has won")));
    }

    #[test]
    fn test_advance_human_turn_sequences() {
        let mut game = GameState::new(Some(1));
        game.start_new_round();

        // Scenario: Next alive after 0 exists
        game.lives = [3, 3, 0, 0];
        game.current_player = 0;
        game.awaiting_human = true;
        let _ = game.advance_after_human_turn();

        // Loop until back to human or finished
        while !game.awaiting_human && !game.finished && !game.pending_new_round {
            let _ = game.continue_automation();
        }

        // Scenario: NO next alive after 0 (Everyone dead)
        let mut game2 = GameState::new(Some(1));
        game2.lives = [0, 0, 0, 0];
        game2.awaiting_human = true;
        let report = game2.advance_after_human_turn();
        assert!(report.draw);
    }

    #[test]
    fn test_impossible_branches() {
        // join_name_list with 0
        assert_eq!(GameState::join_name_list(&[]), "");

        // decrement_life when already 0
        let mut game = GameState::new(None);
        game.lives[1] = 0;
        let mut losses = Vec::new();
        game.decrement_life(1, &mut losses);
        assert!(losses.is_empty());

        // loss_sentence with 0
        assert_eq!(game.loss_sentence(&[]), "");
    }
}
