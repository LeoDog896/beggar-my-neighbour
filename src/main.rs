//! implementation of beggar my neighbour card game

use rayon::prelude::*;
use std::{fmt::{Debug, Display}, collections::VecDeque};
use rand::seq::SliceRandom;
use clap::{Parser, Subcommand};

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
    fn penalty(self) -> usize {
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

        write!(f, "{s}")
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

lazy_static! {
    static ref STATIC_DECK: [Card; 52] = static_deck();
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
    /// Player 1's deck, as a queue (we add to the back and remove from the front)
    p1: VecDeque<Card>,
    /// Player 2's deck, as a queue (we add to the back and remove from the front)
    p2: VecDeque<Card>,
    /// The middle pile, as a vec (we only ever add to it)
    middle: Vec<Card>,
    current_player: Player,
    penalty: usize,
}

struct GameStats {
    turns: u128,
    tricks: u128,
}

impl Game {
    fn random() -> Game {
        let mut rng = rand::thread_rng();
        let mut deck: [Card; DECK_SIZE] = STATIC_DECK.clone();
        deck.shuffle(&mut rng);

        let (p1, p2) = deck.split_at(P_SIZE);
        debug_assert!(p2.len() == P_SIZE);

        let mut p1_queue = VecDeque::with_capacity(DECK_SIZE);
        let mut p2_queue = VecDeque::with_capacity(DECK_SIZE);

        p1_queue.append(&mut p1.to_vec().into());
        p2_queue.append(&mut p2.to_vec().into());

        Game {
            p1: p1_queue,
            p2: p2_queue,
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

    fn from_string(string: &str) -> Game {
        let middle: Vec<Card> = Vec::with_capacity(DECK_SIZE);

        let current_player = Player::P1;
        let penalty = 0;

        let split_string: Vec<&str> = string.split('/').collect();

        let p1 = split_string[0].chars().map(|c| match c {
            'A' => Card::Ace,
            'K' => Card::King,
            'Q' => Card::Queen,
            'J' => Card::Jack,
            '-' => Card::Other,
            _ => panic!("invalid character in string"),
        }).collect();

        let p2 = split_string[1].chars().map(|c| match c {
            'A' => Card::Ace,
            'K' => Card::King,
            'Q' => Card::Queen,
            'J' => Card::Jack,
            '-' => Card::Other,
            _ => panic!("invalid character in string"),
        }).collect();

        Game {
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
        debug_assert!(self.winner().is_none());

        let current_player_deck = match self.current_player {
            Player::P1 => &mut self.p1,
            Player::P2 => &mut self.p2,
        };

        // have the player play a card. we can safely unwrap here because we know the player has cards (otherwise the game would be over)
        let card = current_player_deck.pop_front()
            .unwrap_or_else(|| panic!("{:?} has no cards", self.current_player));

        // regardless if the game currently has penalty, if the player plays a penalty card, the penalty is set and the other player must play
        if card.penalty() > 0 {
            let previous_penalty = self.penalty;
            self.penalty = card.penalty();
            self.middle.push(card);
            self.switch_player();
            return previous_penalty == 0;
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

                for card in self.middle.drain(..) {
                    other_player_deck.push_back(card);
                }

                self.switch_player();
                self.penalty = 0;
            }
            _ => {
                self.middle.push(card);
                self.penalty -= 1;
            }
        };

        false
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
    fn play(&mut self) -> GameStats {
        let mut turns = 0;
        let mut tricks = 0;

        while self.winner().is_none() {
            if self.step() {
                tricks += 1;
            }
            turns += 1;
            if turns > 100_000 {
                println!("{}", detail(self));
                panic!("game took too long");
            }
        }

        GameStats {
            turns,
            tricks,
        }
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        s.push_str("p1: ");
        for card in &self.p1 {
            s.push_str(&format!("{card}"));
        }

        s.push_str("\np2: ");
        for card in &self.p2 {
            s.push_str(&format!("{card}"));
        }

        if !self.middle.is_empty() {
            s.push_str("\nmiddle: ");
            for card in &self.middle {
                s.push_str(&format!("{card}"));
            }
        }

        write!(f, "{s}")
    }
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::with_capacity(DECK_SIZE * 2 + 1);

        for card in &self.p1 {
            s.push_str(&format!("{card}"));
        }

        s.push('/');
        
        for card in &self.p2 {
            s.push_str(&format!("{card}"));
        }

        if self.penalty > 0 {
            s.push_str(&format!("+{}", self.penalty));
        }

        write!(f, "{s}")
    }
}

/// A CLI to play games of beggar my neighbour
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    /// Provide a deck to use instead of a random one
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Random,
    Deck {
        /// The deck to use
        deck: String,
    },
    /// Prints the stats for the longest game
    Record,
    Longest
}

fn detail(game: &mut Game) -> String {
    let mut s = String::new();

    s.push_str(&format!("{game}\n"));
    s.push_str(&format!("stringified: {game:?}\n"));
    let stats = game.play();

    s.push_str("\n");

    s.push_str(&format!("winner: {winner:?}\n", winner = game.winner().unwrap()));
    s.push_str(&format!("turns: {turns}\n", turns = stats.turns));
    s.push_str(&format!("tricks: {tricks}\n", tricks = stats.tricks));

    s
}

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Random => {
            println!("{}", detail(&mut Game::random()));
        }
        Commands::Deck { deck } => {
            println!("{}", detail(&mut Game::from_string(&deck)));
        }
        Commands::Record => {
            println!("{}", detail(&mut Game::from_string("----K---A--Q-A--JJA------J/-----KK---------A-JK-Q-Q-Q")));
        }
        Commands::Longest => {
            let mut best_length = 0;

            loop {
                let game = Game::random();
                let mut playable_game = game.clone();
                let stats = playable_game.play();

                if stats.turns > best_length {
                    best_length = stats.turns;

                    println!("{game}");
                    println!("stringified: {game:?}\n");

                    println!("winner: {winner:?}", winner = playable_game.winner().unwrap());
                    println!("turns: {turns}", turns = stats.turns);
                    println!("tricks: {tricks}", tricks = stats.tricks);

                    println!("-------------------");

                }
            }
        }
    }
}
