#[cfg(test)]
mod tests {
    use beggar_my_neighbour::Game;

    #[test]
    fn world_record_game() {
        let record =
            &mut Game::from_string("---AJ--Q---------QAKQJJ-QK/-----A----KJ-K--------A---");

        let stats = record.play();

        assert_eq!(stats.turns, 8_344);
        assert_eq!(stats.tricks, 1_164);
    }
}
