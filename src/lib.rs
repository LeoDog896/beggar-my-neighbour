//! implementation of beggar my neighbour card game
mod circlebuffer;
mod cursorslice;

use circlebuffer::CircularBuffer;
use cursorslice::CursorSlice;
use std::{
    fmt::{Debug, Display},
    ptr,
};

/// Card is an enum representing 5 different types of cards that are used in beggar my neighbour
/// There are 4 of each (Ace, King, Queen, Jack) and 36 other cards
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Card {
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
    #[inline(always)]
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

pub const DECK_SIZE: usize = 52;

pub fn new_deck() -> [Card; DECK_SIZE] {
    let mut deck = [Card::Other; DECK_SIZE];

    for (i, card) in deck.iter_mut().enumerate() {
        *card = match i {
            0..=3 => Card::Ace,
            4..=7 => Card::King,
            8..=11 => Card::Queen,
            12..=15 => Card::Jack,
            _ => Card::Other,
        }
    }

    deck
}

fn randomize_deck(deck: &mut [Card; DECK_SIZE]) {
    for i in (1..deck.len()).rev() {
        unsafe {
            ptr::swap(
                deck.get_unchecked_mut(i),
                deck.get_unchecked_mut(fastrand::usize(0..=i)),
            );
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Winner {
    P1,
    P2,
    Infinite,
}

#[derive(Clone)]
pub struct Game {
    /// Player 1's deck, as a queue (we add to the back and remove from the front)
    p1: CircularBuffer<Card>,
    /// Player 2's deck, as a queue (we add to the back and remove from the front)
    p2: CircularBuffer<Card>,
    /// The middle pile, as a vec (we only ever add to it)
    middle: CursorSlice<Card, DECK_SIZE>,
    penalty: u8,
}

#[derive(Debug, Copy, Clone)]
pub struct GameStats {
    pub turns: usize,
    pub tricks: usize,
}

impl Game {
    #[must_use]
    pub fn random(deck: &mut [Card; DECK_SIZE]) -> Self {
        const MID: usize = DECK_SIZE / 2;

        // We can just shuffle the original deck since it will be re-shuffled every time
        randomize_deck(deck);

        Self {
            p1: unsafe { CircularBuffer::from_memory(deck.as_ptr(), MID) },
            p2: unsafe { CircularBuffer::from_memory(deck.as_ptr().add(MID), MID) },
            middle: CursorSlice::new(),
            penalty: 0,
        }
    }

    pub fn from_string(string: &str) -> Self {
        let split_string: Vec<&str> = string.split('/').collect();

        // Replace dual bound checks with a single check
        assert!(split_string.len() == 2);

        let p1 = split_string[0].chars().map(Card::from_char).collect();
        let p2 = split_string[1].chars().map(Card::from_char).collect();

        Self {
            p1,
            p2,
            middle: CursorSlice::new(),
            penalty: 0,
        }
    }

    #[must_use]
    pub const fn winner(&self) -> Winner {
        if self.p1.len() == 1 {
            Winner::P2
        } else if self.p2.len() == 1 {
            Winner::P1
        } else {
            Winner::Infinite
        }
    }

    /// Plays out a game of beggar my neighbour, returning how many steps it took
    pub fn play(&mut self) -> GameStats {
        // We can't produce a game thats less than 1 turn long
        // so we can skip some arithmetic
        let mut turns = 1;
        let mut tricks = 0;

        let mut current_player = &mut self.p1;
        let mut other_player = &mut self.p2;

        loop {
            unsafe {
                // We can return early (len = 1) because regardless of the card played, the game is over
                if (*current_player).len() == 1 {
                    break GameStats { turns, tricks };
                }

                // we can safely pop here because we know the player has cards (otherwise the game would be over)
                // *unless current_player.len() == 0, which is impossible we only remove 1 card at a time
                let card = (*current_player).pop_unchecked();
                self.middle.push_unchecked(card);
                turns += 1;

                if card == Card::Other {
                    match self.penalty {
                        0 => std::mem::swap(&mut current_player, &mut other_player),
                        // If the penalty is 1 and the player hasn't played a penalty card, the other player takes all the cards
                        // from the middle and adds them to the beginning of their deck
                        1 => {
                            std::mem::swap(&mut current_player, &mut other_player);

                            (*current_player).push_slice(self.middle.slice());
                            self.middle.clear();

                            self.penalty = 0;
                            if turns > 100_000 {
                                break GameStats { turns, tricks };
                            }
                        }
                        _ => self.penalty -= 1,
                    };
                } else {
                    // regardless if the game currently has penalty,
                    // if the player plays a penalty card,
                    // the penalty is set and the other player must play
                    if self.penalty == 0 {
                        tricks += 1;
                    }
                    self.penalty = card.penalty();
                    std::mem::swap(&mut current_player, &mut other_player);
                }
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

    fn assert_game(game: &str, turns: usize, tricks: usize) {
        let game = &mut Game::from_string(game);

        let stats = game.play();

        assert_eq!(stats.turns, turns);
        assert_eq!(stats.tricks, tricks);
    }

    #[test]
    fn world_record_games() {
        assert_game(
            "---AJ--Q---------QAKQJJ-QK/-----A----KJ-K--------A---",
            8_344,
            1_164,
        );
        assert_game(
            "K-KK----K-A-----JAA--Q--J-/---Q---Q-J-----J------AQ--",
            7_157,
            1_007,
        );
        assert_game(
            "A-QK------Q----KA-----J---/-JAK----A--Q----J---QJ--K-",
            6_913,
            960,
        )
    }
}
