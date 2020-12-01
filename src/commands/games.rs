use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::*;
use serenity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
enum MoveResult {
    Win,
    Tie,
    Invalid,
    Continue,
}

struct TTTField {
    field: [[Option<u8>; 3]; 3],
}

impl TTTField {
    fn new() -> Self {
        Self {
            field: Default::default(),
        }
    }
    fn is_tied(&self) -> bool {
        self.is_filled() && !self.winner().is_some()
    }
    fn is_filled(&self) -> bool {
        self.field.iter().flatten().all(|e| e.is_some())
    }
    fn winner(&self) -> Option<u8> {
        let f = &self.field;
        for a in 0..3 {
            let (mut row_full, mut col_full) = (f[a][0].is_some(), f[0][a].is_some());
            for b in 0..3 {
                row_full &= f[a][b] == f[a][0];
                col_full &= f[b][a] == f[0][a];
            }
            if row_full {
                return f[a][0];
            } else if col_full {
                return f[0][a];
            }
        }

        let (mut diag1, mut diag2) = (f[0][0].is_some(), f[0][2].is_some());
        for i in 0..3 {
            diag1 &= f[i][i] == f[0][0];
            diag2 &= f[2 - i][i] == f[2][0];
        }
        if diag1 {
            self.field[0][0]
        } else if diag2 {
            self.field[0][2]
        } else {
            None
        }
    }
    fn make_move(&mut self, x: usize, y: usize, player: u8) -> MoveResult {
        if x >= 3 || y >= 3 {
            MoveResult::Invalid
        } else if self.field[x][y].is_some() {
            MoveResult::Invalid
        } else {
            self.field[x][y] = Some(player);
            if self.winner() == Some(player) {
                MoveResult::Win
            } else if self.is_filled() {
                MoveResult::Tie
            } else {
                MoveResult::Continue
            }
        }
    }
}

fn number_emoji(num: usize) -> ReactionType {
    ReactionType::Unicode(format!("{}\u{fe0f}\u{20e3}", num))
}

#[command]
async fn tictactoe(ctx: &Context, prompt: &Message) -> CommandResult {
    let mut players = prompt.mentions.clone();
    players.push(prompt.author.clone());

    if players.len() != 2 {
        prompt
            .channel_id
            .say(&ctx.http, "You need to tag another person to play against!")
            .await?;
        return Ok(());
    }

    let shapes = ['X', '@'];

    let msg_content = |field: &TTTField, turn| {
        let mut msg = String::from("TicTacToe!\n");
        msg += &format!(
            "{} plays `{}`, {} plays `{}`\n",
            players[0].mention(),
            shapes[0],
            players[1].mention(),
            shapes[1]
        );

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
                let ch = match field.field[row][col] {
                    None => std::char::from_digit((3 * row + col) as u32, 10).unwrap(),
                    Some(p) => shapes[p as usize],
                };
                grid[2 * row][1 + 4 * col] = ch;
            }
        }

        // print grid
        msg += "```\n";
        for line in grid.iter() {
            msg += &format!("{}\n", line.iter().collect::<String>());
        }
        msg += "```\n";

        if let Some(winner) = field.winner() {
            msg += &format!("{} won!\n", players[winner as usize].mention());
        } else if field.is_tied() {
            msg += &format!("It's a tie!\n");
        } else {
            msg += &format!("`{}` plays next.\n", shapes[turn]);
        }
        msg
    };

    let mut field = TTTField::new();

    let mut message = prompt
        .channel_id
        .say(&ctx.http, msg_content(&field, 0))
        .await?;

    for i in 0..9 {
        message.react(&ctx.http, number_emoji(i)).await?;
    }

    'game: loop {
        for turn in 0..2 {
            message
                .edit(&ctx.http, |m| m.content(msg_content(&field, turn)))
                .await?;

            let play = loop {
                let reaction = message.await_reaction(&ctx.shard).await;
                if let Some(reaction) = reaction {
                    let reaction = reaction.as_inner_ref();
                    if Some(players[turn].id) != reaction.user_id {
                        // it is not the current player who has reacted
                        continue;
                    }

                    if let Some(num) = (0..9).map(number_emoji).position(|e| e == reaction.emoji) {
                        let row = num / 3;
                        let col = num % 3;

                        let state = field.make_move(row, col, turn as u8);
                        if state != MoveResult::Invalid {
                            break state;
                        }
                    }
                }
            };

            if play == MoveResult::Win || play == MoveResult::Tie {
                break 'game;
            }
        }
    }

    message
        .edit(&ctx.http, |m| m.content(msg_content(&field, 0)))
        .await?;

    Ok(())
}

struct UltimateGame {
    field: [[Option<u8>; 9]; 9],
    cell: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum GameStatus {
    Running,
    Tie,
    Invalid,
    Win(u8),
}

impl GameStatus {
    fn is_finished(&self) -> bool {
        match self {
            Self::Tie => true,
            Self::Win(_) => true,
            _ => false,
        }
    }
}

fn field_status(field: &[Option<u8>; 9]) -> GameStatus {
    let mut win_combos = vec![[0, 4, 8], [2, 4, 6]];
    for i in 0..3 {
        let i3 = 3 * i;
        win_combos.push([i, i + 3, i + 6]);
        win_combos.push([i3, i3 + 1, i3 + 2]);
    }
    for combo in win_combos.iter() {
        if field[combo[0]].is_some() && (0..3).all(|i| field[combo[i]] == field[combo[0]]) {
            return GameStatus::Win(field[combo[0]].unwrap());
        }
    }
    if field.iter().all(|e| e.is_some()) {
        GameStatus::Tie
    } else {
        GameStatus::Running
    }
}

impl UltimateGame {
    fn new() -> Self {
        Self {
            field: Default::default(),
            cell: 0,
        }
    }
    fn flatten_xy(x: usize, y: usize) -> usize {
        3 * y + x
    }
    fn status(&self) -> GameStatus {
        let mut wins = [None; 9];
        for i in 0..9 {
            if let GameStatus::Win(p) = field_status(&self.field[i]) {
                wins[i] = Some(p);
            }
        }
        field_status(&wins)
    }
    fn make_move(&mut self, pos: usize, player: u8) -> GameStatus {
        if self.field[self.cell][pos].is_some() {
            return GameStatus::Invalid;
        }

        self.field[self.cell][pos] = Some(player);
        self.cell = pos;

        // find next playable field
        for i in 0..9 {
            let cell_i = (self.cell + i) % 9;
            if field_status(&self.field[cell_i]) == GameStatus::Running {
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
                let xf = 2 + 19 * x;
                let yf = 1 + 10 * y;

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
                let sub_field = self.field[Self::flatten_xy(x, y)];
                for x in 0..3 {
                    for y in 0..3 {
                        let idx = Self::flatten_xy(x, y);
                        if let Some(p) = sub_field[idx] {
                            let x = xf + 2 + 4 * x;
                            let y = yf + 1 + 2 * y;
                            field[y][x] = symbols[p as usize];
                        }
                    }
                }

                field[yf - 1][xf - 1] = numbers[Self::flatten_xy(x, y)];

                // draw selection
                if self.cell == Self::flatten_xy(x, y) {
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

#[command]
#[only_in(guilds)]
#[description(
    "You play on a 3x3 grid of tictactoe fields.
Where you move in the small field determines which field your opponent is going to play in next.
A win in a small field counts as a mark on the big field.
You win if you have three in a row, column or diagonal in the big field."
)]
async fn ultimate(ctx: &Context, prompt: &Message) -> CommandResult {
    let mut players = prompt.mentions.clone();
    players.push(prompt.author.clone());

    if players.len() != 2 {
        prompt
            .channel_id
            .say(&ctx.http, "You need to tag another person to play against!")
            .await?;
        return Ok(());
    }

    let shapes = ['X', '@'];
    let mut game = UltimateGame::new();

    let msg_content = |game: &UltimateGame, turn: usize| {
        let mut msg = format!(
            "Ultimate TicTacToe!\n{} plays with `{}`, {} plays with `{}`\n",
            players[0].mention(),
            shapes[0],
            players[1].mention(),
            shapes[1]
        );

        msg += &format!("```\n{}\n```\n", game.draw(&shapes));

        match game.status() {
            GameStatus::Win(p) => msg += &format!("{} won!\n", players[p as usize].mention()),
            GameStatus::Tie => msg += "It's a tie!\n",
            GameStatus::Running => msg += &format!("Next turn: `{}`", shapes[turn]),
            _ => unreachable!(),
        };

        msg
    };

    let mut message = prompt
        .channel_id
        .say(&ctx.http, msg_content(&game, 0))
        .await?;

    for i in 1..10 {
        message.react(&ctx.http, number_emoji(i)).await?;
    }

    'game: for turn in (0..2).cycle() {
        message
            .edit(&ctx.http, |m| m.content(msg_content(&game, turn)))
            .await?;

        let play = loop {
            let reaction = message.await_reaction(&ctx.shard).await;
            if let Some(reaction) = reaction {
                let reaction = reaction.as_inner_ref();
                if Some(players[turn].id) != reaction.user_id {
                    // it is not the current player who has reacted
                    continue;
                }

                if let Some(num) = (1..10).map(number_emoji).position(|e| e == reaction.emoji) {
                    let state = game.make_move(num, turn as u8);
                    if state != GameStatus::Invalid {
                        break state;
                    }
                }
            }
        };

        if play.is_finished() {
            break 'game;
        }
    }

    message
        .edit(&ctx.http, |m| m.content(msg_content(&game, 0)))
        .await?;

    Ok(())
}
