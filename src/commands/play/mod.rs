use crate::ser::*;
use crate::{prelude::*, tryc};
use rusqlite::{params, Result};
use std::time::*;

mod connect4;
mod elo;
mod mcts;
mod minimax;
mod pentago;
mod random_ai;
mod runner;
mod tictactoe;
mod ultimate;
mod util;
use minimax::*;
use random_ai::*;
use runner::GameRunner;
use util::*;

macro_rules! make_games {
    ($($(#[$meta:meta])* game $name:ident ($struct:expr, $timeout:expr); )*) => {
        mod game {
            use super::*;
            $(
                #[command]
                #[only_in(guilds)]
                #[bucket("game")]
                #[usage = "[casual] <enemy_player>"]
                $(
                    #[$meta]
                )*
                async fn $name(ctx: &Context, prompt: &Message) -> CommandResult {
                    let mut runner = GameRunner::new(ctx, prompt, $struct, stringify!($name), $timeout as f64).await?;
                    runner.run(ctx).await
                }
            )*

            #[group]
            #[help_available]
            #[prefix = "play"]
            #[commands($(
                $name
            ),*)]
            #[allow(unused)]
            pub struct Games;

        }

        mod score {
            use super::*;
            $(
                #[command]
                #[only_in(guilds)]
                async fn $name(ctx: &Context, msg: &Message) -> CommandResult {
                    fn title<T, G: PvpGame<T>>(_: &G) -> &'static str { G::title() }
                    leaderboard(ctx, msg, stringify!($name), title(&$struct)).await
                }
            )*

            #[group]
            #[help_available]
            #[prefix = "leaderboard"]
            #[commands($(
                $name
            ),*)]
            #[allow(unused)]
            pub struct Leaderboard;
        }

        pub use game::GAMES_GROUP;
        pub use score::LEADERBOARD_GROUP;
    };
}

make_games! {
    #[description("The classic 3x3 game without strategy.")]
    game tictactoe (tictactoe::TTTField::default(), 60.0);

    #[description(
        "You play on a 3x3 grid of tictactoe fields.
Where you move in the small field determines which field your opponent is going to play in next.
If that targeted field is already occupied (won/lost/tied), the field with the next bigger index is chosen.
A win in a small field counts as a mark on the big field.
You win if you have three in a row, column or diagonal in the big field."
    )]
    game ultimate(ultimate::UltimateGame::new(), 60.0);

    #[description("The classic Connect Four game.
A person wins if four discs of the same color are arranged in a row, column, or diagonal.")]
    game connect4(connect4::Connect4::default(), 60.0);

    #[description("The game is played on a 6×6 board divided into four 3×3 sub-boards (or quadrants). Taking turns, the two players place a marble of their color onto an unoccupied space on the board, and then rotate one of the sub-boards by 90 degrees either clockwise or anti-clockwise. A player wins by getting five of their marbles in a vertical, horizontal or diagonal row (either before or after the sub-board rotation in their move). **Important**: the game is played by text. Type `XYSR` to make a move. `X` and `Y` are the location of your next move. `S` is the number of the subfield you want to rotate. `R` is the direction of the rotation of the subfield (**A**nticlockwise or **C**lockwise). Example: `314A` (place marble on (3, 1), rotate field 4 90 degress anticlockwise).")]
    game pentago(pentago::Pentago::default(), 60.0);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameState {
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

/// All functions a game must possess
pub trait PvpGame<T> {
    /// Title of the game
    fn title() -> &'static str;
    /// Input Method
    fn input() -> Box<dyn InputMethod<T> + Send + Sync>;
    /// Display the current board in the discord message
    fn draw(&self) -> String;
    /// Make a game move
    fn make_move(&mut self, action: T, person: usize) -> GameState;
    fn status(&self) -> GameState;
    fn winner(&self) -> Option<usize> {
        if let GameState::Win(winner) = self.status() {
            Some(winner)
        } else {
            None
        }
    }
    fn ai() -> Option<Box<dyn AiPlayer<T, Self> + Send + Sync>> {
        None
    }
    fn possible_moves(&self, _player: usize) -> Vec<T> {
        Vec::new()
    }
    fn figures() -> Vec<String>;
    fn is_empty(&self) -> bool;
}

pub trait AiPlayer<T, G: PvpGame<T>> {
    fn make_move(&mut self, game: &G, player_id: usize) -> T;
}

#[async_trait]
pub trait InputMethod<Input: 'static> {
    async fn prepare(&self, _: &Context, _: &Message) -> CommandResult {
        Ok(())
    }
    async fn receive_input(
        &self,
        ctx: &Context,
        msg: &Message,
        player: &UserId,
    ) -> CommandResult<Input>;
}

struct ReactionInput(pub Vec<ReactionType>);
#[async_trait]
impl InputMethod<usize> for ReactionInput {
    async fn prepare(&self, ctx: &Context, msg: &Message) -> CommandResult {
        for r in self.0.iter() {
            msg.react(ctx, r.clone()).await?;
        }
        Ok(())
    }
    async fn receive_input(
        &self,
        ctx: &Context,
        msg: &Message,
        player: &UserId,
    ) -> CommandResult<usize> {
        let reaction = msg
            .await_reaction(ctx)
            .author_id(*player.as_u64())
            .removed(true)
            .timeout(Duration::from_secs_f64(10.0))
            .await
            .ok_or("no reaction")?;

        let idx = self
            .0
            .iter()
            .position(|e| *e == reaction.as_inner_ref().emoji)
            .ok_or("no fitting reaction")?;

        Ok(idx)
    }
}

struct TextInput<T>(pub Box<dyn Send + Sync + Fn(&str) -> CommandResult<T>>);
#[async_trait]
impl<T: 'static + Send + Sync> InputMethod<T> for TextInput<T> {
    async fn receive_input(
        &self,
        ctx: &Context,
        msg: &Message,
        player: &UserId,
    ) -> CommandResult<T> {
        let msg = msg
            .channel_id
            .await_reply(ctx)
            .author_id(*player.as_u64())
            .timeout(Duration::from_secs_f64(10.0))
            .await
            .ok_or("no message")?;

        let parsed = (self.0)(&msg.content);

        if parsed.is_ok() {
            msg.delete(ctx).await.ok();
        }

        parsed
    }
}

async fn leaderboard(ctx: &Context, msg: &Message, game: &str, game_name: &str) -> CommandResult {
    let server = format!("{}", msg.guild_id.ok_or("not sent in a guild")?);

    let players = {
        let db = db()?;
        let mut stmt = db.prepare(&format!(
            "SELECT player, elo FROM {} WHERE server=?1",
            elo_table(&game)
        ))?;
        let players_iter = stmt.query_map(params!(server), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })?;

        let mut players = Vec::new();
        for entry in players_iter {
            let (player, elo) = tryc!(entry.ok());
            let player = UserId(player.parse::<u64>().unwrap());
            players.push((player, elo));
        }

        players.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        players
    };

    let mut leaderboard = Vec::new();

    for (idx, (user, elo)) in players.into_iter().enumerate() {
        let rank = rank_string(idx + 1);
        let points = format!("{:0>4}", elo as i64);

        let user = match user.to_user(ctx).await {
            Ok(user) => user.mention().to_string(),
            Err(_) => String::from("<invalid user>"),
        };

        leaderboard.push(format!("`{} {}  ` {}\n", rank, points, user));
    }

    let leaderboard = split_into_fields(&leaderboard, "This leaderboard is empty");

    msg.ereply(ctx, |e| {
        e.title(format!("{} Leaderboard", game_name));
        e.description(&leaderboard[0])
    })
    .await?;

    Ok(())
}

fn rank_string(rank: usize) -> String {
    let suffix = match rank % 10 {
        _ if (rank / 10) % 10 == 1 => "th",
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    };
    format!("{: >3}{}", rank, suffix)
}

fn log_game(
    game: &str,
    server: u64,
    player_id: &[u64],
    moves: &[u8],
    winner: Option<usize>,
) -> Result<()> {
    let player1 = format!("{}", player_id[0]);
    let player2 = format!("{}", player_id[1]);
    let server = format!("{}", server);
    let result = winner.map_or(0, |win| win as u8 + 1);
    db()?.execute(
        &format!(
            "INSERT INTO {} (server, player1, player2, moves, result) VALUES (?1, ?2, ?3, ?4, ?5);",
            games_table(game)
        ),
        params!(server, player1, player2, &moves, result),
    )?;
    Ok(())
}

fn games_table(game: &str) -> String {
    format!("{}_games", game)
}

fn elo_table(game: &str) -> String {
    format!("{}_elo", game)
}

fn create_tables(game: &str) -> Result<()> {
    db()?.execute(
        &format!("CREATE TABLE IF NOT EXISTS {} (server TEXT, player1 TEXT, player2 TEXT, moves BLOB, result INTEGER);", games_table(game)),
        params!(),
    )?;
    db()?.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {} (server TEXT, player TEXT, elo REAL);",
            elo_table(game)
        ),
        params!(),
    )?;
    Ok(())
}
