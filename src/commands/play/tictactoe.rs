use super::*;

trait Draw {
    fn draw(&self, player_symbols: &[char]) -> String;
}

pub type TTTField = [Option<usize>; 9];

impl Draw for TTTField {
    fn draw(&self, player_symbols: &[char]) -> String {
        let mut grid = vec![vec![' '; 11]; 5];

        // vertical lines
        for x in 0..2 {
            let x = 3 + 4 * x;
            for y in 0..5 {
                grid[y][x] = '|';
            }
        }

        // horizontal lines
        for y in 0..2 {
            let y = 1 + 2 * y;
            for x in 0..11 {
                grid[y][x] = if grid[y][x] == '|' { '+' } else { '-' };
            }
        }

        for row in 0..3 {
            for col in 0..3 {
                let idx = flatten_xy(col, row);
                let ch = match self[idx] {
                    None => std::char::from_digit(1 + (3 * row + col) as u32, 10).unwrap(),
                    Some(p) => player_symbols[p],
                };
                grid[2 * row][1 + 4 * col] = ch;
            }
        }

        // playing field string
        let mut playing_field = String::new();
        for line in grid.iter() {
            playing_field += &format!("{}\n", line.iter().collect::<String>());
        }
        playing_field
    }
}

impl PvpGame for TTTField {
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
    fn reactions() -> Vec<ReactionType> {
        (1..10).map(number_emoji).collect()
    }
    fn edit_message<'e>(&self, m: &'e mut EditMessage, ctx: &GameContext) -> &'e mut EditMessage {
        let title = String::from("TicTacToe!\n");
        let subtitle = format!(
            "{} plays `{}`\n{} plays `{}`\n",
            ctx.players[0].mention(),
            ctx.shapes[0],
            ctx.players[1].mention(),
            ctx.shapes[1]
        );

        let playing_field = format!("```\n{}\n```\n", self.draw(&ctx.shapes));

        let footer_text = {
            if let Some(winner) = self.winner() {
                format!("{} won!\n", ctx.players[winner].mention())
            } else if self.status() == GameState::Tie {
                format!("It's a tie!\n")
            } else {
                format!("`{}` plays next.\n", ctx.shapes[ctx.turn])
            }
        };

        m.embed(|e| {
            let title = title.clone();
            let subtitle = subtitle.clone();
            let playing_field = playing_field.clone();
            let footer_text = footer_text.clone();
            e.title(title);
            e.description(subtitle);
            e.field("Field", playing_field, false);
            e.field("Game Status", footer_text, false);
            e
        })
    }
}

pub fn flatten_xy(x: usize, y: usize) -> usize {
    3 * y + x
}
