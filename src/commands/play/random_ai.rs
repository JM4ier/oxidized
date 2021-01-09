use super::*;
use rand::prelude::*;
use std::marker::*;

#[derive(Default)]
pub struct RandomPlayer<G> {
    _phantom: PhantomData<G>,
}

impl<G: PvpGame + Clone> AiPlayer<G> for RandomPlayer<G> {
    fn make_move(&mut self, game: &G, player_id: usize) -> usize {
        let mut valid_moves = (0..G::reactions().len())
            .filter(|&m| game.clone().make_move(m, player_id) != GameState::Invalid)
            .collect::<Vec<_>>();
        valid_moves.shuffle(&mut thread_rng());
        valid_moves[0]
    }
}
