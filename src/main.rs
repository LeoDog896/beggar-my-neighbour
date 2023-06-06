use beggar_my_neighbour::Game;
use clap::{Parser, Subcommand};
use rand::{rngs::SmallRng, SeedableRng};
use rayon::prelude::ParallelIterator;
use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    },
};

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
    Longest,
}

fn detail(game: &mut Game) -> String {
    let mut s = String::new();

    s.push_str(&format!("{game}\n"));
    s.push_str(&format!("stringified: {game:?}\n"));
    let stats = game.play();

    s.push('\n');

    s.push_str(&format!(
        "winner: {winner:?}\n",
        winner = game.winner().unwrap()
    ));
    s.push_str(&format!("turns: {turns}\n", turns = stats.turns));
    s.push_str(&format!("tricks: {tricks}\n", tricks = stats.tricks));

    s
}

fn main() {
    let args = Args::parse();
    let rng: Mutex<SmallRng> = Mutex::new(SmallRng::from_entropy());
    match args.command {
        Commands::Random => {
            println!("{}", detail(&mut Game::random(&mut *rng.lock().unwrap())));
        }
        Commands::Deck { deck } => {
            println!("{}", detail(&mut Game::from_string(&deck)));
        }
        Commands::Record => {
            println!(
                "{}",
                detail(&mut Game::from_string(
                    "---AJ--Q---------QAKQJJ-QK/-----A----KJ-K--------A---"
                ))
            );
        }
        Commands::Longest => {
            let best_length: AtomicUsize = AtomicUsize::new(0);

            rayon::iter::repeat(())
                .for_each(|_| {
                    let game = Game::random(&mut *rng.lock().unwrap());
                    let mut playable_game = game.clone();
                    let stats = playable_game.play();

                    let length = best_length.load(Ordering::Relaxed);

                    if stats.turns > length {
                        best_length.store(stats.turns, Ordering::Relaxed);

                        println!("{game}");
                        println!("stringified: {game:?}\n");

                        println!(
                            "winner: {winner:?}",
                            winner = playable_game.winner().unwrap()
                        );
                        println!("turns: {turns}", turns = stats.turns);
                        println!("tricks: {tricks}", tricks = stats.tricks);

                        println!("-------------------");
                    }
                });
        }
    }
}
