use super::*;

pub trait MinimaxAi<G: PvpGame + Clone> {
    fn rate(&self, board: &G, player: usize) -> f64;
    fn depth(&self) -> usize;
    fn default_move(&self) -> usize;
}

pub fn minimax<G: PvpGame + Clone, M: MinimaxAi<G>>(
    mm: &M,
    board: &G,
    player: usize,
    depth: usize,
) -> (f64, usize) {
    if board.is_empty() {
        (0.5, mm.default_move())
    } else if let GameState::Win(winner) = board.status() {
        if winner == player {
            (1.0, 0)
        } else {
            (0.0, 0)
        }
    } else if GameState::Tie == board.status() {
        (0.5, 0)
    } else if depth == 0 {
        (mm.rate(board, player), 0)
    } else {
        let moves = G::reactions().len();
        let mut max = 0.0_f64;
        let mut best_move = 0;

        for mov in 0..moves {
            let mut eboard = board.clone();
            let status = eboard.make_move(mov, player);
            if status != GameState::Invalid {
                let score = 1.0 - minimax(mm, &eboard, 1 - player, depth - 1).0;
                if score > max {
                    max = score;
                    best_move = mov;
                }
            }
        }
        (max, best_move)
    }
}

impl<G: PvpGame + Clone, M: MinimaxAi<G>> AiPlayer<G> for M {
    fn make_move(&self, board: &G, id: usize) -> usize {
        minimax(self, board, id, self.depth() + 1).1
    }
}
