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
#[repr(u8)]
#[derive(Debug, Copy, Clone)]
enum Card {
    /// Penalty card, play 4
    Ace = 4,
    /// Penalty card, play 3
    King = 3,
    /// Penalty card, play 2
    Queen = 2,
    /// Penalty card, play 1
    Jack = 1,
    Other = 0,
}

impl Card {
    const fn penalty(self) -> u8 {
        self as u8
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

const DECK_SIZE: usize = 52;

fn static_deck() -> [Card; DECK_SIZE] {
    (0..DECK_SIZE)
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
    static ref STATIC_DECK: Mutex<[Card; DECK_SIZE]> = Mutex::new(static_deck());
}

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
    penalty: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct GameStats {
    pub turns: usize,
    pub tricks: usize,
}

impl Game {
    pub fn random<R>(rng: &mut R) -> Self
    where
        R: Rng + ?Sized,
    {
        // We can just shuffle the original deck since it will be re-shuffled every time
        let mut deck: [Card; DECK_SIZE] = *STATIC_DECK.lock().unwrap();
        deck.shuffle(rng);

        let (p1, p2) = deck.split_at(DECK_SIZE / 2);
        debug_assert!(p2.len() == DECK_SIZE / 2);

        Self {
            p1: unsafe { SliceFifo::from_slice(p1) },
            p2: unsafe { SliceFifo::from_slice(p2) },
            middle: ClearVec::new(),
            penalty: 0,
        }
    }

    fn switch_player(&mut self, ptr: *mut SliceFifo<Card, DECK_SIZE>) -> *mut SliceFifo<Card, DECK_SIZE> {
        // check if current_player is the same as p1
        if std::ptr::eq(ptr, &self.p1) {
            &mut self.p2
        } else {
            &mut self.p1
        }
    }

    pub fn from_string(string: &str) -> Self {
        let middle = ClearVec::new();

        let penalty = 0;

        let split_string: Vec<&str> = string.split('/').collect();

        let p1 = split_string[0].chars().map(Card::from_char).collect();
        let p2 = split_string[1].chars().map(Card::from_char).collect();

        Self {
            p1,
            p2,
            middle,
            penalty,
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

        // TODO this is a hack. we should probably just drop current_player from struct
        // since this is a silly raw pointer
        let mut current_player: *mut SliceFifo<Card, 52> = &mut self.p1;

        loop {
            unsafe {
                // have the player play a card. we can safely pop here because we know the player has cards (otherwise the game would be over)
                let card = (*current_player).pop_unchecked();

                if (*current_player).is_empty() {
                    break GameStats {
                        turns: turns + 1,
                        tricks,
                    }
                }

                self.middle.push_unchecked(card);

                // regardless if the game currently has penalty, if the player plays a penalty card, the penalty is set and the other player must play
                if card.penalty() > 0 {
                    let previous_penalty = self.penalty;
                    self.penalty = card.penalty();
                    current_player = self.switch_player(current_player);
                    turns += 1;
                    if previous_penalty == 0 {
                        tricks += 1;
                    }
                    continue;
                }

                match self.penalty {
                    0 => current_player = self.switch_player(current_player),
                    // If the penalty is 1 and the player hasn't played a penalty card, the other player takes all the cards
                    // from the middle and adds them to the beginning of their deck
                    1 => {
                        current_player = self.switch_player(current_player);

                        (*current_player).push_slice(self.middle.slice());
                        self.middle.clear();

                        self.penalty = 0;
                    }
                    _ => self.penalty -= 1,
                };

                turns += 1;
            }
            if turns > 100_000 {
                break GameStats { turns, tricks };
            }
        }
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

#[cfg(test)]
mod tests {
    use super::Game;

    #[test]
    fn world_record_game() {
        let record =
            &mut Game::from_string("---AJ--Q---------QAKQJJ-QK/-----A----KJ-K--------A---");

        let stats = record.play();

        assert_eq!(stats.turns, 8_344);
        assert_eq!(stats.tricks, 1_164);
    }
}
