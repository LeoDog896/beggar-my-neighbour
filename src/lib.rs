//! implementation of beggar my neighbour card game

mod clearvec;
mod slicefifo;

use clearvec::ClearVec;
use rand::{seq::SliceRandom, Rng};
use slicefifo::SliceFifo;
use std::{
    fmt::{Debug, Display},
    sync::Mutex,
};

#[macro_use]
extern crate lazy_static;

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

impl Card {
    const fn penalty(self) -> usize {
        match self {
            Self::Ace => 4,
            Self::King => 3,
            Self::Queen => 2,
            Self::Jack => 1,
            Self::Other => 0,
        }
    }

    fn from_char(c: char) -> Self {
        match c {
            'A' => Self::Ace,
            'K' => Self::King,
            'Q' => Self::Queen,
            'J' => Self::Jack,
            '-' => Self::Other,
            _ => panic!("invalid character in string"),
        }
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Ace => "A",
            Self::King => "K",
            Self::Queen => "Q",
            Self::Jack => "J",
            Self::Other => "-",
        };

        write!(f, "{s}")
    }
}

fn static_deck() -> [Card; 52] {
    (0..52)
        .map(|i| match i {
            0..=3 => Card::Ace,
            4..=7 => Card::King,
            8..=11 => Card::Queen,
            12..=15 => Card::Jack,
            _ => Card::Other,
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

lazy_static! {
    static ref STATIC_DECK: Mutex<[Card; 52]> = Mutex::new(static_deck());
}

const DECK_SIZE: usize = 52;
const P_SIZE: usize = DECK_SIZE / 2;

#[derive(Debug, Copy, Clone)]
pub enum Player {
    P1,
    P2,
}

#[derive(Clone)]
pub struct Game {
    /// Player 1's deck, as a queue (we add to the back and remove from the front)
    p1: SliceFifo<Card, DECK_SIZE>,
    /// Player 2's deck, as a queue (we add to the back and remove from the front)
    p2: SliceFifo<Card, DECK_SIZE>,
    /// The middle pile, as a vec (we only ever add to it)
    middle: ClearVec<Card, DECK_SIZE>,
    current_player: Player,
    penalty: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct GameStats {
    pub turns: usize,
    pub tricks: usize,
}

impl Game {
    pub fn random<R>(rng: &mut R) -> Self where R: Rng + ?Sized {
        // We can just shuffle the original deck since it will be re-shuffled every time
        let mut deck: [Card; DECK_SIZE] = *STATIC_DECK.lock().unwrap();
        deck.shuffle(rng);

        let (p1, p2) = deck.split_at(P_SIZE);
        debug_assert!(p2.len() == P_SIZE);

        Self {
            p1: SliceFifo::from_slice(p1),
            p2: SliceFifo::from_slice(p2),
            middle: ClearVec::new(),
            current_player: Player::P1,
            penalty: 0,
        }
    }

    fn switch_player(&mut self) {
        self.current_player = match self.current_player {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        }
    }

    pub fn from_string(string: &str) -> Self {
        let middle = ClearVec::new();

        let current_player = Player::P1;
        let penalty = 0;

        let split_string: Vec<&str> = string.split('/').collect();

        let p1 = split_string[0].chars().map(Card::from_char).collect();

        let p2 = split_string[1].chars().map(Card::from_char).collect();

        Self {
            p1,
            p2,
            middle,
            current_player,
            penalty,
        }
    }

    /// Emulates a step of beggar my neighbour as a player,
    /// modifying the game state
    ///
    /// Returns true if there was a trick, false otherwise
    fn step(&mut self) -> bool {
        unsafe {
            debug_assert!(self.winner().is_none());

            let current_player_deck = match self.current_player {
                Player::P1 => &mut self.p1,
                Player::P2 => &mut self.p2,
            };

            // have the player play a card. we can safely pop here because we know the player has cards (otherwise the game would be over)
            let card = current_player_deck.pop_unchecked();

            // regardless if the game currently has penalty, if the player plays a penalty card, the penalty is set and the other player must play
            if card.penalty() > 0 {
                let previous_penalty = self.penalty;
                self.penalty = card.penalty();
                self.middle.push_unchecked(card);
                self.switch_player();
                return previous_penalty == 0;
            }

            match self.penalty {
                0 => {
                    self.middle.push_unchecked(card);
                    self.switch_player();
                }
                // If the penalty is 1 and the player hasn't played a penalty card, the other player takes all the cards
                // from the middle and adds them to the beginning of their deck
                1 => {
                    self.middle.push_unchecked(card);

                    let other_player_deck = match self.current_player {
                        Player::P1 => &mut self.p2,
                        Player::P2 => &mut self.p1,
                    };

                    other_player_deck.push_slice(self.middle.as_slice());
                    self.middle.clear();

                    self.switch_player();
                    self.penalty = 0;
                }
                _ => {
                    self.middle.push_unchecked(card);
                    self.penalty -= 1;
                }
            };

            false
        }
    }

    pub const fn winner(&self) -> Option<Player> {
        if self.p1.is_empty() {
            Some(Player::P2)
        } else if self.p2.is_empty() {
            Some(Player::P1)
        } else {
            None
        }
    }

    /// Plays out a game of beggar my neighbour, returning how many steps it took
    pub fn play(&mut self) -> GameStats {
        let mut turns = 0;
        let mut tricks = 0;

        while self.winner().is_none() {
            if self.step() {
                tricks += 1;
            }
            turns += 1;
            if turns > 100_000 {
                return GameStats { turns, tricks };
            }
        }

        GameStats { turns, tricks }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        s.push_str("p1: ");
        for card in self.p1.iter() {
            s.push_str(&format!("{card}"));
        }

        s.push_str("\np2: ");
        for card in self.p2.iter() {
            s.push_str(&format!("{card}"));
        }

        if !self.middle.is_empty() {
            s.push_str("\nmiddle: ");
            for card in self.middle.iter() {
                s.push_str(&format!("{card}"));
            }
        }

        write!(f, "{s}")
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::with_capacity(DECK_SIZE + 4);

        for card in self.p1.iter() {
            s.push_str(&format!("{card}"));
        }

        s.push('/');

        for card in self.p2.iter() {
            s.push_str(&format!("{card}"));
        }

        if self.penalty > 0 {
            s.push_str(&format!("+{}", self.penalty));
        }

        write!(f, "{s}")
    }
}