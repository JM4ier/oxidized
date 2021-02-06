use super::*;
use rand::prelude::*;
use std::marker::*;

#[derive(Default)]
pub struct RandomPlayer<G> {
    _phantom: PhantomData<G>,
}

impl<G: PvpGame<usize> + Clone> AiPlayer<usize, G> for RandomPlayer<G> {
    fn make_move(&mut self, game: &G, player_id: usize) -> usize {
        let mut valid_moves = game.possible_moves(player_id);
        valid_moves.shuffle(&mut thread_rng());
        valid_moves[0]
    }
}
