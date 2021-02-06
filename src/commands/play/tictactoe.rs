use super::*;

pub type TTTField = [Option<usize>; 9];

pub const EMPTY: &str = "⬛";
pub const BORDER: &str = "⬜";
pub const TIE: &str = "🟦";

impl PvpGame<usize> for TTTField {
    fn title() -> &'static str {
        "Tic Tac Toe"
    }
    fn is_empty(&self) -> bool {
        *self == Self::default()
    }
    fn status(&self) -> GameState {
        let mut win_combos = vec![[0, 4, 8], [2, 4, 6]];
        for i in 0..3 {
            let i3 = 3 * i;
            win_combos.push([i, i + 3, i + 6]);
            win_combos.push([i3, i3 + 1, i3 + 2]);
        }
        for combo in win_combos.iter() {
            if self[combo[0]].is_some() && (0..3).all(|i| self[combo[i]] == self[combo[0]]) {
                return GameState::Win(self[combo[0]].unwrap());
            }
        }
        if self.iter().all(|e| e.is_some()) {
            GameState::Tie
        } else {
            GameState::Running
        }
    }
    fn make_move(&mut self, idx: usize, person: usize) -> GameState {
        if idx >= self.len() {
            GameState::Invalid
        } else if self[idx].is_some() {
            GameState::Invalid
        } else {
            self[idx] = Some(person);
            self.status()
        }
    }
    fn input() -> Box<dyn InputMethod<usize> + Send + Sync> {
        Box::new(ReactionInput((1..10).map(number_emoji).collect()))
    }
    fn figures() -> Vec<String> {
        vec![String::from("🟥"), String::from("🟨")]
    }
    fn draw(&self) -> String {
        let mut playing_field = String::new();

        for y in 0..3 {
            for iy in 0..3 {
                for x in 0..3 {
                    let fig = Self::figures();

                    let idx = flatten_xy(x, y);
                    for ix in 0..3 {
                        let sym = match self[idx] {
                            Some(p) => &fig[p],
                            None if ix == iy && ix == 1 => &NUMBERS[idx + 1],
                            None => EMPTY,
                        };
                        playing_field += sym;
                    }

                    if x < 2 {
                        playing_field += BORDER;
                    } else {
                        playing_field += "\n";
                    }
                }
            }
            if y < 2 {
                for _ in 0..11 {
                    playing_field += BORDER
                }
                playing_field += "\n";
            }
        }

        playing_field
    }
    fn ai() -> Option<Box<dyn AiPlayer<usize, Self> + Send + Sync>> {
        Some(Box::new(Minimax(TTTAI)))
    }
    fn possible_moves(&self, _: usize) -> Vec<usize> {
        (0..9).filter(|&i| self[i].is_none()).collect()
    }
}

pub struct TTTAI;

impl MinimaxAi<TTTField> for TTTAI {
    fn rate(&self, _: &TTTField, _: usize) -> f64 {
        0.0
    }
    fn default_move(&self) -> usize {
        0
    }
    fn depth(&self) -> usize {
        9
    }
}

pub fn flatten_xy(x: usize, y: usize) -> usize {
    3 * y + x
}
