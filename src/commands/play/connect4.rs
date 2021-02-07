use super::*;
use crate::cart;

#[derive(Default, PartialEq, Eq)]
pub struct Connect4 {
    field: [[Option<usize>; ROWS]; COLS],
}

const ROWS: usize = 6;
const COLS: usize = 7;

impl Connect4 {
    fn filled(&self) -> bool {
        self.field.iter().all(|col| col.iter().all(Option::is_some))
    }
}

impl PvpGame<usize> for Connect4 {
    fn input() -> Box<dyn InputMethod<usize> + Send + Sync> {
        Box::new(ReactionInput((0..COLS).map(number_emoji).collect()))
    }
    fn draw(&self) -> String {
        let mut drawing = String::new();
        let figs = Self::figures();

        for y in (0..ROWS).rev() {
            for x in 0..COLS {
                drawing += match self.field[x][y] {
                    Some(player) => &figs[player],
                    None => tictactoe::EMPTY,
                };
            }
            drawing += "\n";
        }
        for x in 0..COLS {
            drawing += &NUMBERS[x];
        }
        drawing
    }
    fn make_move(&mut self, idx: usize, person: usize) -> GameState {
        assert!(idx < COLS);
        for entry in self.field[idx].iter_mut() {
            if entry.is_none() {
                *entry = Some(person);
                return self.status();
            }
        }
        // this column is already filled to the top
        return GameState::Invalid;
    }
    fn status(&self) -> GameState {
        util::n_in_a_row(ROWS, COLS, &|x, y| self.field[x][y], 4)
    }
    fn title() -> &'static str {
        "Connect Four"
    }
    fn figures() -> Vec<String> {
        tictactoe::TTTField::figures()
    }
    fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}
