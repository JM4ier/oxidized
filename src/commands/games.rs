use crate::tryc;
use serenity::builder::*;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::{channel::*, user::*};
use serenity::prelude::*;

fn number_emoji(num: usize) -> ReactionType {
    ReactionType::Unicode(format!("{}\u{fe0f}\u{20e3}", num))
}

#[command]
async fn tictactoe(ctx: &Context, prompt: &Message) -> CommandResult {
    pvp_game(ctx, prompt, SmallField::default()).await
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
    Win(usize),
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

type SmallField = [Option<usize>; 9];

trait Status {
    fn status(&self) -> GameState;
    fn winner(&self) -> Option<usize> {
        if let GameState::Win(winner) = self.status() {
            Some(winner)
        } else {
            None
        }
    }
}

trait Move {
    fn make_move(&mut self, idx: usize, person: usize) -> GameState;
}
trait Draw {
    fn draw(&self, player_symbols: &[char]) -> String;
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
}

impl Draw for SmallField {
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
                    None => std::char::from_digit((3 * row + col) as u32, 10).unwrap(),
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

impl PvpGame for SmallField {
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

impl Move for UltimateGame {
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

impl PvpGame for UltimateGame {
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
                            field[y][x] = symbols[p];
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

trait PvpGame: Status + Move {
    fn edit_message<'e>(&self, m: &'e mut EditMessage, ctx: &GameContext) -> &'e mut EditMessage;
    fn reactions() -> Vec<ReactionType>;
}

struct GameContext {
    players: Vec<User>,
    shapes: Vec<char>,
    turn: usize,
}

impl GameContext {
    fn next_turn(&mut self) {
        self.turn = 1 - self.turn;
    }
}

/// Plays a PvpGame where two people play against each other
async fn pvp_game<G: PvpGame>(ctx: &Context, prompt: &Message, mut game: G) -> CommandResult {
    let mut players = prompt.mentions.clone();
    players.push(prompt.author.clone());

    if players.len() != 2 {
        prompt
            .channel_id
            .say(&ctx.http, "You need to tag another person to play against!")
            .await?;
        return Ok(());
    }

    // create message and react
    let mut message = prompt.channel_id.say(&ctx.http, "Loading Game...").await?;
    for r in G::reactions() {
        message.react(&ctx.http, r).await?;
    }

    let mut game_ctx = GameContext {
        players,
        shapes: vec!['X', '@'],
        turn: 0,
    };

    loop {
        message
            .edit(&ctx.http, |m| game.edit_message(m, &game_ctx))
            .await?;

        let play = loop {
            let reaction = message.await_reaction(&ctx.shard).await;
            let reaction = tryc!(reaction);
            let reaction = reaction.as_inner_ref();

            // check player who has reacted
            if Some(game_ctx.players[game_ctx.turn].id) != reaction.user_id {
                continue;
            }

            // if it is one of the given emojis, try to make that move
            let idx = tryc!(G::reactions().into_iter().position(|e| e == reaction.emoji));

            let state = game.make_move(idx, game_ctx.turn);
            if state != GameState::Invalid {
                break state;
            }
        };

        if play.is_finished() {
            break;
        } else {
            game_ctx.next_turn();
        }
    }

    message
        .edit(&ctx.http, |m| game.edit_message(m, &game_ctx))
        .await?;
    Ok(())
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
    pvp_game(ctx, prompt, UltimateGame::new()).await
}
