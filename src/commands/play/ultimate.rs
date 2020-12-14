use super::tictactoe::*;
use super::*;

#[derive(Clone, PartialEq, Eq, Default)]
pub struct UltimateGame {
    field: [TTTField; 9],
    cell: usize,
}

impl PvpGame for UltimateGame {
    fn is_empty(&self) -> bool {
        *self == Self::default()
    }
    fn status(&self) -> GameState {
        let mut wins = [None; 9];
        for i in 0..9 {
            if let GameState::Win(p) = self.field[i].status() {
                wins[i] = Some(p);
            }
        }
        wins.status()
    }
    fn edit_message<'e>(&self, m: &'e mut EditMessage, ctx: &GameContext) -> &'e mut EditMessage {
        let mut msg = format!(
            "Ultimate TicTacToe!\n{} plays with `{}`, {} plays with `{}`\n",
            ctx.players[0].mention(),
            ctx.shapes[0],
            ctx.players[1].mention(),
            ctx.shapes[1]
        );

        msg += &format!("```\n{}\n```\n", self.draw(&ctx.shapes));

        match self.status() {
            GameState::Win(p) => msg += &format!("{} won!\n", ctx.players[p].mention()),
            GameState::Tie => msg += "It's a tie!\n",
            GameState::Running => msg += &format!("Next turn: `{}`", ctx.shapes[ctx.turn]),
            _ => unreachable!(),
        };

        m.content(msg)
    }
    fn reactions() -> Vec<ReactionType> {
        (1..10).map(number_emoji).collect()
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
}

impl UltimateGame {
    pub fn new() -> Self {
        Self {
            field: Default::default(),
            cell: 0,
        }
    }
    fn draw(&self, symbols: &[char]) -> String {
        let numbers = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];
        let selection = '*';
        let mut field = vec![vec![' '; 55]; 29];

        // draw vertical big lines
        for x in 0..2 {
            let x = 17 + 19 * x;
            for y in 0..field.len() {
                field[y][x + 0] = '|';
                field[y][x + 1] = '|';
            }
        }

        // draw horizontal big lines
        for y in 0..2 {
            let y = 9 + 10 * y;
            for x in 0..field[0].len() {
                field[y][x] = '=';
            }
        }

        // draw small fields
        for x in 0..3 {
            for y in 0..3 {
                // x,y offset
                let xf = 2 + 19 * x;
                let yf = 1 + 10 * y;

                // draw number of field in top left corner
                field[yf - 1][xf - 1] = numbers[flatten_xy(x, y)];

                let status = self.field[flatten_xy(x, y)].status();
                if status.is_finished() {
                    if let GameState::Win(winner) = status {
                        for x in xf..=(xf + 12) {
                            for y in yf..=(yf + 6) {
                                field[y][x] = symbols[winner];
                            }
                        }
                    } else {
                        let midy = yf + 3;
                        let midx = xf + 5;
                        field[midy][midx + 0] = 'T';
                        field[midy][midx + 1] = 'I';
                        field[midy][midx + 2] = 'E';
                    }
                    continue;
                }

                // vertical lines
                for x in 0..2 {
                    let x = xf + 4 * (x + 1);
                    for y in 0..5 {
                        let y = yf + y + 1;
                        field[y][x] = '|';
                    }
                }

                // horizontal lines
                for y in 0..2 {
                    let y = yf + 2 * (y + 1);
                    for x in 0..11 {
                        let x = xf + x + 1;
                        field[y][x] = '-';
                    }
                }

                // draw numbers or symbols
                let sub_field = self.field[flatten_xy(x, y)];
                for x in 0..3 {
                    for y in 0..3 {
                        let idx = flatten_xy(x, y);
                        if let Some(p) = sub_field[idx] {
                            let x = xf + 2 + 4 * x;
                            let y = yf + 1 + 2 * y;
                            field[y][x] = symbols[p];
                        }
                    }
                }

                // draw selection
                if self.cell == flatten_xy(x, y) {
                    let xe = xf + 12;
                    let ye = yf + 6;
                    for y in yf..=ye {
                        field[y][xf] = selection;
                        field[y][xe] = selection;
                    }
                    for x in (xf..=xe).step_by(2) {
                        field[yf][x] = selection;
                        field[ye][x] = selection;
                    }
                }
            }
        }

        field
            .into_iter()
            .map(|line| format!("{}\n", line.iter().collect::<String>()))
            .fold(String::new(), |a, b| a + &b)
    }
}
