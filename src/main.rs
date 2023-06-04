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

enum Player {
    P1,
    P2,
}

struct GameState {
    player: Player,
    won: Option<Player>,
}

struct Game {
    p1: [Option<Card>; DECK_SIZE],
    p2: [Option<Card>; DECK_SIZE],
    middle: [Option<Card>; DECK_SIZE],
    state: GameState,
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

        let (p1_orig, p2_orig) = deck.split_at(P_SIZE);
        assert!(p2_orig.len() == P_SIZE);

        let mut p1: [Option<Card>; DECK_SIZE] = [None; DECK_SIZE];
        let mut p2: [Option<Card>; DECK_SIZE] = [None; DECK_SIZE];
        let middle: [Option<Card>; DECK_SIZE] = [None; DECK_SIZE];

        for i in 0..P_SIZE {
            p1[i] = Some(p1_orig[i]);
            p2[i] = Some(p2_orig[i]);
        }

        Game {
            p1,
            p2,
            middle,
            state: GameState {
                player: Player::P1,
                won: None,
            },
        }
    }

    /// Emulates a step of beggar my neighbour as a player,
    /// modifying the game state
    fn step(&mut self, player: Player) {
        
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
        for i in 0..DECK_SIZE {
            if let Some(c) = self.p1[i] {
                s.push_str(&format!("{:?} ", c));
            }
        }
        s.push_str("\n");

        s.push_str("p2: ");
        for i in 0..DECK_SIZE {
            if let Some(c) = self.p2[i] {
                s.push_str(&format!("{:?} ", c));
            }
        }
        s.push_str("\n");

        s.push_str("middle: ");
        for i in 0..DECK_SIZE {
            if let Some(c) = self.middle[i] {
                s.push_str(&format!("{:?} ", c));
            }
        }
        s.push_str("\n");

        write!(f, "{}", s)
    }
}

fn main() {
    let game = Game::random();

    println!("{:?}", game);
}
