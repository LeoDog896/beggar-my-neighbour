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

impl Card {
    fn penalty(&self) -> usize {
        match self {
            Card::Ace => 4,
            Card::King => 3,
            Card::Queen => 2,
            Card::Jack => 1,
            Card::Other => 0,
        }
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Card::Ace => "A",
            Card::King => "K",
            Card::Queen => "Q",
            Card::Jack => "J",
            Card::Other => "-",
        };

        write!(f, "{}", s)
    }
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

#[derive(Clone)]
struct Game {
    p1: Vec<Card>,
    p2: Vec<Card>,
    middle: Vec<Card>,
    current_player: Player,
    penalty: usize,
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
            penalty: 0,
        }
    }

    fn switch_player(&mut self) {
        self.current_player = match self.current_player {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        }
    }

    /// Emulates a step of beggar my neighbour as a player,
    /// modifying the game state
    fn step(&mut self) {
        if self.winner().is_some() {
            return;
        }

        let current_player_deck = match self.current_player {
            Player::P1 => &mut self.p1,
            Player::P2 => &mut self.p2,
        };

        // have the player play a card. we can safely unwrap here because we know the player has cards (otherwise the game would be over)
        let card = current_player_deck
            .pop()
            .expect(format!("{:?} has no cards", self.current_player).as_str());

        // regardless if the game currently has penalty, if the player plays a penalty card, the penalty is set and the other player must play
        if card.penalty() > 0 {
            self.penalty = card.penalty();
            self.middle.push(card);
            self.switch_player();
            return;
        }

        match self.penalty {
            0 => {
                self.middle.push(card);
                self.switch_player();
            }
            // If the penalty is 1 and the player hasn't played a penalty card, the other player takes all the cards
            // from the middle and adds them to the beginning of their deck (in reverse order)
            1 => {
                self.middle.push(card);

                let other_player_deck = match self.current_player {
                    Player::P1 => &mut self.p2,
                    Player::P2 => &mut self.p1,
                };

                other_player_deck.splice(
                    0..0,
                    self.middle
                        .drain(..)
                        .rev()
                        .collect::<Vec<Card>>()
                        .into_iter(),
                );

                self.switch_player();
                self.penalty = 0;
            }
            _ => {
                self.middle.push(card);
                self.penalty -= 1;
            }
        }
    }

    fn winner(&self) -> Option<Player> {
        if self.p1.is_empty() {
            Some(Player::P2)
        } else if self.p2.is_empty() {
            Some(Player::P1)
        } else {
            None
        }
    }

    /// Plays out a game of beggar my neighbour, returning how many steps it took
    fn play(&mut self) -> u128 {
        let mut steps = 0;

        while self.winner().is_none() {
            self.step();
            steps += 1;
        }

        steps
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        s.push_str("p1: ");
        for card in &self.p1 {
            s.push_str(&format!("{}", card));
        }

        s.push_str("\np2: ");
        for card in &self.p2 {
            s.push_str(&format!("{}", card));
        }

        if !self.middle.is_empty() {
            s.push_str("\nmiddle: ");
            for card in &self.middle {
                s.push_str(&format!("{}", card));
            }
        }

        write!(f, "{}", s)
    }
}

fn main() {
    let mut game = Game::random();

    println!("{:?}", game);

    let steps = game.play();

    println!();
    println!("{:?}", game);
    println!("winner: {:?}, steps: {}", game.winner(), steps);
}
