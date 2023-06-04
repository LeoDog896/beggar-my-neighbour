//! implementation of beggar my neighbour card game

use std::fmt::{Debug, Display};

use rand::{seq::SliceRandom, Rng};

/// Card is an enum representing 5 different types of cards that are used in beggar my neighbour
/// There are 4 of each (Ace, King, Queen, Jack) and 36 other cards
#[derive(Debug, Copy, Clone)]
enum Card {
    /// Penalty card, play 4
    Ace,
    /// Penalty card, play 3
    King,
    /// Penalty card, play 2
    Queen,
    /// Penalty card, play 1
    Jack,
    Other,
}

fn static_deck() -> [Card; 52] {
    let mut deck: [Card; 52] = [Card::Other; 52];

    for i in 0..52 {
        deck[i] = match i {
            0..=3 => Card::Ace,
            4..=7 => Card::King,
            8..=11 => Card::Queen,
            12..=15 => Card::Jack,
            _ => Card::Other,
        };
    }

    deck
}

const DECK_SIZE: usize = 52;
const P_SIZE: usize = DECK_SIZE / 2;

#[derive(Debug, Copy, Clone)]
enum Player {
    P1,
    P2,
}

struct Game {
    p1: Vec<Card>,
    p2: Vec<Card>,
    middle: Vec<Card>,
    current_player: Player,
    won: Option<Player>,
    penalty: usize,
}

struct GameStats {
    tricks: usize,
    cards_played: usize,
}

impl Game {
    fn random() -> Game {
        let mut rng = rand::thread_rng();
        let mut deck: [Card; DECK_SIZE] = static_deck();
        deck.shuffle(&mut rng);

        let (p1, p2) = deck.split_at(P_SIZE);
        assert!(p2.len() == P_SIZE);

        Game {
            p1: p1.to_vec(),
            p2: p2.to_vec(),
            middle: Vec::with_capacity(DECK_SIZE),
            current_player: Player::P1,
            won: None,
            penalty: 0,
        }
    }

    /// Emulates a step of beggar my neighbour as a player,
    /// modifying the game state
    fn step(&mut self) {
        let current_player_deck = match self.current_player {
            Player::P1 => &mut self.p1,
            Player::P2 => &mut self.p2,
        };

        // have the player play a card
        let card = current_player_deck.pop().unwrap();
    }

    /// Plays out a game of beggar my neighbour, returning statistics about the game
    fn play(&self) -> GameStats {
        let mut game = self.clone();

        // TODO: implement
        GameStats {
            tricks: 0,
            cards_played: 0,
        }
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        s.push_str("p1: ");
        for card in &self.p1 {
            s.push_str(&format!("{:?}, ", card));
        }

        s.push_str("\np2: ");
        for card in &self.p2 {
            s.push_str(&format!("{:?}, ", card));
        }

        s.push_str("\nmiddle: ");
        for card in &self.middle {
            s.push_str(&format!("{:?}, ", card));
        }

        write!(f, "{}", s)
    }
}

fn main() {
    let game = Game::random();

    println!("{:?}", game);
}
