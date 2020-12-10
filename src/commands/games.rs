use serenity::builder::*;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::*;
use serenity::prelude::*;
use serenity::utils::*;

fn number_emoji(num: usize) -> ReactionType {
    ReactionType::Unicode(format!("{}\u{fe0f}\u{20e3}", num))
}

fn infer<T, F: for<'a> Fn(&'a mut T) -> &'a mut T>(f: F) -> F {
    f
}

#[command]
async fn tictactoe(ctx: &Context, prompt: &Message) -> CommandResult {
    let mut players = prompt.mentions.clone();
    players.push(prompt.author.clone());

    if players.len() != 2 {
        prompt
            .channel_id
            .send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title("Tic Tac Toe");
                    e.colour(Colour::RED);
                    e.description("You need to tag another person to play against!")
                })
            })
            .await?;
        return Ok(());
    }

    let shapes = ['X', '@'];

    let msg_content = |field: &SmallField, turn| {
        let title = String::from("TicTacToe!\n");
        let subtitle = format!(
            "{} plays `{}`\n{} plays `{}`\n",
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
                let idx = flatten_xy(col, row);
                let ch = match field[idx] {
                    None => std::char::from_digit((3 * row + col) as u32, 10).unwrap(),
                    Some(p) => shapes[p as usize],
                };
                grid[2 * row][1 + 4 * col] = ch;
            }
        }

        // playing field string
        let mut playing_field = String::from("```\n");
        for line in grid.iter() {
            playing_field += &format!("{}\n", line.iter().collect::<String>());
        }
        playing_field += "```\n";

        let footer_text = {
            if let Some(winner) = field.winner() {
                format!("{} won!\n", players[winner as usize].mention())
            } else if field.status() == GameState::Tie {
                format!("It's a tie!\n")
            } else {
                format!("`{}` plays next.\n", shapes[turn])
            }
        };

        infer(move |e: &mut CreateEmbed| {
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
    };

    let mut field = SmallField::default();

    let mut game_msg = prompt
        .channel_id
        .send_message(&ctx.http, |m| m.content("Loading..."))
        .await?;

    for i in 0..9 {
        game_msg.react(&ctx.http, number_emoji(i)).await?;
    }

    'game: loop {
        for turn in 0..2 {
            game_msg
                .edit(&ctx.http, |m| {
                    m.content("").embed(|e| msg_content(&field, turn)(e))
                })
                .await?;

            let play = loop {
                let reaction = game_msg.await_reaction(&ctx.shard).await;
                if let Some(reaction) = reaction {
                    let reaction = reaction.as_inner_ref();
                    if Some(players[turn].id) != reaction.user_id {
                        // it is not the current player who has reacted
                        continue;
                    }

                    if let Some(num) = (0..9).map(number_emoji).position(|e| e == reaction.emoji) {
                        let state = field.make_move(num, turn as u8);
                        if state != GameState::Invalid {
                            break state;
                        }
                    }
                }
            };

            if play.is_finished() {
                break 'game;
            }
        }
    }

    game_msg
        .edit(&ctx.http, |m| m.embed(|e| msg_content(&field, 0)(e)))
        .await?;

    Ok(())
}

struct UltimateGame {
    field: [SmallField; 9],
    cell: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum GameState {
    Running,
    Tie,
    Invalid,
    Win(u8),
}

impl GameState {
    fn is_finished(&self) -> bool {
        match self {
            Self::Tie => true,
            Self::Win(_) => true,
            _ => false,
        }
    }
}

type SmallField = [Option<u8>; 9];

trait Status {
    fn status(&self) -> GameState;
    fn winner(&self) -> Option<u8> {
        if let GameState::Win(winner) = self.status() {
            Some(winner)
        } else {
            None
        }
    }
}

trait Move {
    fn make_move(&mut self, idx: usize, person: u8) -> GameState;
}

impl Status for SmallField {
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
}

impl Move for SmallField {
    fn make_move(&mut self, idx: usize, person: u8) -> GameState {
        if idx >= self.len() {
            GameState::Invalid
        } else if self[idx].is_some() {
            GameState::Invalid
        } else {
            self[idx] = Some(person);
            self.status()
        }
    }
}

impl Status for UltimateGame {
    fn status(&self) -> GameState {
        let mut wins = [None; 9];
        for i in 0..9 {
            if let GameState::Win(p) = self.field[i].status() {
                wins[i] = Some(p);
            }
        }
        wins.status()
    }
}

fn flatten_xy(x: usize, y: usize) -> usize {
    3 * y + x
}

impl UltimateGame {
    fn new() -> Self {
        Self {
            field: Default::default(),
            cell: 0,
        }
    }
    fn make_move(&mut self, pos: usize, player: u8) -> GameState {
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
                let sub_field = self.field[flatten_xy(x, y)];
                for x in 0..3 {
                    for y in 0..3 {
                        let idx = flatten_xy(x, y);
                        if let Some(p) = sub_field[idx] {
                            let x = xf + 2 + 4 * x;
                            let y = yf + 1 + 2 * y;
                            field[y][x] = symbols[p as usize];
                        }
                    }
                }

                field[yf - 1][xf - 1] = numbers[flatten_xy(x, y)];

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
            GameState::Win(p) => msg += &format!("{} won!\n", players[p as usize].mention()),
            GameState::Tie => msg += "It's a tie!\n",
            GameState::Running => msg += &format!("Next turn: `{}`", shapes[turn]),
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
                    if state != GameState::Invalid {
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
