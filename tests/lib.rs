#![feature(test)]
extern crate test;

#[cfg(test)]
mod tests {
    use beggar_my_neighbour::Game;
    use test::Bencher;
    
    #[bench]
    fn bench_run_game(b: &mut Bencher) {
        b.iter(|| {
            let record =
                &mut Game::from_string("---AJ--Q---------QAKQJJ-QK/-----A----KJ-K--------A---");

            record.play();
        });
    }
}