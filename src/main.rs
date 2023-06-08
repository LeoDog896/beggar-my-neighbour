use beggar_my_neighbour::Game;
use clap::{Parser, Subcommand};
use rand::rngs::ThreadRng;
use rayon::prelude::ParallelIterator;
use std::{
    fmt::Debug,
    sync::atomic::{AtomicUsize, Ordering},
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

fn game_header(game: &Game) -> String {
    let mut s = String::new();

    s.push_str(&format!("{game}\n"));
    s.push_str(&format!("stringified: {game:?}\n"));

    s
}

fn detail(game: &mut Game) -> String {
    let mut s = String::new();

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
    match args.command {
        Commands::Random => {
            let mut game = Game::random(&mut ThreadRng::default());
            println!("{}", game_header(&game));
            println!("{}", detail(&mut game));
        }
        Commands::Deck { deck } => {
            let mut game = Game::from_string(&deck);
            println!("{}", game_header(&game));
            println!("{}", detail(&mut game));
        }
        Commands::Record => {
            let mut game: &mut Game = &mut Game::from_string(
                "---AJ--Q---------QAKQJJ-QK/-----A----KJ-K--------A---"
            );
            println!(
                "{}",
                game_header(&game)
            );
            println!(
                "{}",
                detail(&mut game)
            );
        }
        Commands::Longest => {
            let best_length: AtomicUsize = AtomicUsize::new(0);

            rayon::iter::repeat(()).for_each(|_| {
                let game = Game::random(&mut ThreadRng::default());
                let mut playable_game = game.clone();
                let stats = playable_game.play();

                let length = best_length.load(Ordering::Relaxed);

                if stats.turns > length {
                    best_length.store(stats.turns, Ordering::Relaxed);

                    println!("{}", game_header(&game));

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
