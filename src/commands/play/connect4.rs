use super::*;

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

macro_rules! cart {
    ($a:expr, $b:expr) => {
        $a.flat_map(move |a| $b.map(move |b| (a, b)))
    };
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
        for (x, y) in cart!(0..COLS, 0..ROWS) {
            let color = match self.field[x][y] {
                Some(color) => color,
                None => continue,
            };
            'dir: for (dx, dy) in cart!(-1..=1, -1..=1) {
                if dx == 0 && dy == 0 {
                    continue;
                }
                for i in 1..4 {
                    let x = i * dx + x as isize;
                    let y = i * dy + y as isize;
                    if x < 0 || y < 0 || x >= COLS as _ || y >= ROWS as _ {
                        continue 'dir;
                    }
                    if self.field[x as usize][y as usize] != Some(color) {
                        continue 'dir;
                    }
                }
                return GameState::Win(color);
            }
        }
        if self.filled() {
            GameState::Tie
        } else {
            GameState::Running
        }
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
