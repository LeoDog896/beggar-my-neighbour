use beggar_my_neighbour::{new_deck, Card, Game, DECK_SIZE};
use clap::{Parser, Subcommand};
use indoc::printdoc;
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
    /// Plays a random deck
    Random,
    /// Plays a specific deck
    Deck {
        /// The deck to use
        deck: String,
    },
    /// Prints the stats for the longest game
    Record,
    /// Attempts to find a long game
    Longest {
        /// How many games to play
        /// Don't specify if you want to play forever
        #[arg(short, long)]
        games: Option<usize>,
    },
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

    s.push_str(&format!("winner: {winner:?}\n", winner = game.winner()));
    s.push_str(&format!("turns: {turns}\n", turns = stats.turns));
    s.push_str(&format!("tricks: {tricks}\n", tricks = stats.tricks));

    s
}

fn random_game(best_length: &AtomicUsize, deck: &mut [Card; DECK_SIZE]) {
    let game = Game::random(deck);
    let mut playable_game = game.clone();
    let stats = playable_game.play();

    let length = best_length.load(Ordering::Relaxed);

    if stats.turns > length {
        best_length.store(stats.turns, Ordering::Relaxed);
        printdoc!(
            "{header}

            winner: {winner:?}
            turns: {turns}
            tricks: {tricks}
            -------------------
            ",
            winner = playable_game.winner(),
            turns = stats.turns,
            tricks = stats.tricks,
            header = game_header(&game),
        );
    }
}

fn main() {
    let args = Args::parse();
    match args.command {
        Commands::Random => {
            let mut game = Game::random(&mut new_deck());
            println!("{}", game_header(&game));
            println!("{}", detail(&mut game));
        }
        Commands::Deck { deck } => {
            let mut game = Game::from_string(&deck);
            println!("{}", game_header(&game));
            println!("{}", detail(&mut game));
        }
        Commands::Record => {
            let game: &mut Game =
                &mut Game::from_string("---AJ--Q---------QAKQJJ-QK/-----A----KJ-K--------A---");
            println!("{}", game_header(game));
            println!("{}", detail(game));
        }
        Commands::Longest { games: total_games } => {
            static BEST_LENGTH: AtomicUsize = AtomicUsize::new(0);
            static GAMES: AtomicUsize = AtomicUsize::new(0);

            let threads = std::thread::available_parallelism().unwrap();

            let mut handles: Vec<_> = (0..threads.into())
                .map(|_| {
                    std::thread::spawn(move || {
                        let mut deck = new_deck();
                        loop {
                            random_game(&BEST_LENGTH, &mut deck);
                            let games = GAMES.fetch_add(1, Ordering::Relaxed);

                            if let Some(total_games) = total_games {
                                if games >= total_games {
                                    std::process::exit(0);
                                }
                            }
                        }
                    })
                })
                .collect();

            for handle in handles.drain(..) {
                handle.join().unwrap();
            }
        }
    }
}
