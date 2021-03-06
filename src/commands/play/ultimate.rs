use super::tictactoe::*;
use super::*;

#[derive(Clone, PartialEq, Eq, Default)]
pub struct UltimateGame {
    field: [TTTField; 9],
    cell: usize,
}

impl PvpGame<usize> for UltimateGame {
    fn title() -> &'static str {
        "Ultimate Tic Tac Toe"
    }
    fn figures() -> Vec<String> {
        TTTField::figures()
    }
    fn ai() -> Option<Box<dyn AiPlayer<usize, Self> + Send + Sync>> {
        Some(Box::new(RandomPlayer::<Self>::default()))
    }
    fn possible_moves(&self, player: usize) -> Vec<usize> {
        self.field[self.cell].possible_moves(player)
    }
    fn is_empty(&self) -> bool {
        *self == Self::default()
    }
    fn status(&self) -> GameState {
        let mut wins = [None; 9];
        for i in 0..9 {
            wins[i] = match self.field[i].status() {
                GameState::Win(p) => Some(p),
                GameState::Tie => Some(42),
                _ => None,
            }
        }
        wins.status()
    }
    fn input() -> Box<dyn InputMethod<usize> + Send + Sync> {
        Box::new(ReactionInput((1..10).map(number_emoji).collect()))
    }
    fn make_move(&mut self, pos: usize, player: usize) -> GameState {
        if self.field[self.cell][pos].is_some() {
            return GameState::Invalid;
        }

        self.field[self.cell][pos] = Some(player);
        self.cell = pos;

        // find next playable field
        for i in 0..9 {
            let cell_i = (self.cell + i) % 9;
            if self.field[cell_i].status() == GameState::Running {
                self.cell = cell_i;
                break;
            }
        }

        // don't display a selection box if the game is finished
        if self.status().is_finished() {
            self.cell = 10;
        }

        self.status()
    }
    fn draw(&self) -> String {
        let mut field = String::new();
        for y in 0..3 {
            for iy in 0..3 {
                for x in 0..3 {
                    for ix in 0..3 {
                        let o = flatten_xy(x, y);
                        let i = flatten_xy(ix, iy);
                        let fig = Self::figures();
                        let sym = match self.field[o].status() {
                            GameState::Win(p) => &fig[p],
                            GameState::Tie => TIE,
                            _ => match self.field[o][i] {
                                Some(p) => &fig[p],
                                _ if o == self.cell => &NUMBERS[i + 1],
                                _ => EMPTY,
                            },
                        };
                        field += sym;
                    }
                    if x < 2 {
                        field += BORDER;
                    }
                }
                field += "\n";
            }
            if y < 2 {
                for _ in 0..3 * 4 - 1 {
                    field += BORDER;
                }
            }
            field += "\n";
        }
        field
    }
}

impl UltimateGame {
    pub fn new() -> Self {
        Self {
            field: Default::default(),
            cell: 0,
        }
    }
}

pub struct UltimateMMAI;

impl MinimaxAi<UltimateGame> for UltimateMMAI {
    fn rate(&self, board: &UltimateGame, id: usize) -> f64 {
        let mut sum = 0.0;
        for field in board.field.iter() {
            sum += super::minimax::minimax(&TTTAI, field, id, 9).0;
        }
        sum
    }
    fn depth(&self) -> usize {
        0
    }
    fn default_move(&self) -> usize {
        0
    }
}
