use crate::ser::*;
use crate::{prelude::*, tryc};
use lazy_static::*;
use rusqlite::{params, Result};
use std::time::*;

mod connect4;
mod elo;
mod mcts;
mod minimax;
mod random_ai;
mod runner;
mod tictactoe;
mod ultimate;
use minimax::*;
use random_ai::*;

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
                    pvp_game(ctx, prompt, $struct, stringify!($name), $timeout as f64).await
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
                    fn title<G: PvpGame>(_: &G) -> &'static str { G::title() }
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
pub trait PvpGame {
    /// All possible reactions to this game
    fn reactions() -> Vec<ReactionType>;
    /// Display the current board in the discord message
    fn draw(&self, ctx: &GameContext) -> String;
    /// Make a game move
    ///
    /// `idx` is the index of the reacted emoji
    fn make_move(&mut self, idx: usize, person: usize) -> GameState;
    fn status(&self) -> GameState;
    fn winner(&self) -> Option<usize> {
        if let GameState::Win(winner) = self.status() {
            Some(winner)
        } else {
            None
        }
    }
    fn ai() -> Option<Box<dyn AiPlayer<Self> + Send + Sync>> {
        None
    }
    fn title() -> &'static str;
    fn figures() -> Vec<String>;
    fn is_empty(&self) -> bool;
}

pub trait AiPlayer<G: PvpGame> {
    fn make_move(&mut self, game: &G, player_id: usize) -> usize;
}

pub struct GameContext {
    players: Vec<User>,
    turn: usize,
}

impl GameContext {
    fn next_turn(&mut self) {
        self.turn = 1 - self.turn;
    }
}

#[async_trait]
trait InputMethod {
    type Input;
    async fn prepare(&self, _: &Context, _: &Message) -> CommandResult {
        Ok(())
    }
    async fn receive_input(
        &self,
        ctx: &Context,
        msg: &Message,
        player: &UserId,
    ) -> CommandResult<Self::Input>;
}

struct ReactionInput(Vec<ReactionType>);
#[async_trait]
impl InputMethod for ReactionInput {
    type Input = usize;
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
    ) -> CommandResult<Self::Input> {
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

struct TextInput<T>(dyn Send + Sync + Fn(&str) -> CommandResult<T>);
#[async_trait]
impl<T> InputMethod for TextInput<T> {
    type Input = T;
    async fn receive_input(
        &self,
        ctx: &Context,
        msg: &Message,
        player: &UserId,
    ) -> CommandResult<Self::Input> {
        let msg = msg
            .channel_id
            .await_reply(ctx)
            .author_id(*player.as_u64())
            .timeout(Duration::from_secs_f64(10.0))
            .await
            .ok_or("no message")?;

        (self.0)(&msg.content)
    }
}

/// Plays a PvpGame where two people play against each other
async fn pvp_game<G: PvpGame + Send + Sync>(
    ctx: &Context,
    prompt: &Message,
    mut game: G,
    game_name: &str,
    timeout: f64,
) -> CommandResult {
    let cmds = commands();
    let cmd = cmds
        .iter()
        .filter(|c| c.options.names.contains(&game_name))
        .next()
        .unwrap();

    create_tables(game_name)?;

    let mut moves = Vec::new();

    let mut players = prompt.mentions.clone();
    players.push(prompt.author.clone());

    if players.len() != 2 {
        prompt
            .ereply(ctx, |e| {
                e.title("Error");
                e.description("You need to tag another person to play against!");
                e.color(Color::RED)
            })
            .await?;
        return Ok(());
    }

    // check if the player wants to play against AI
    let mut ai_player_id = None;
    for (i, player) in players.iter().enumerate() {
        if player.id == ctx.cache.current_user().await.id {
            ai_player_id = Some(i);
        }
    }

    // check if the game even supports AI
    if ai_player_id.is_some() && G::ai().is_none() {
        prompt
            .ereply(ctx, |e| {
                e.title("Error");
                e.description("This game doesn't support AI players.");
                e.color(Color::RED)
            })
            .await?;
        return Ok(());
    }

    // see if this is meant to be played competitively, default to true if no argument
    let compete = ai_player_id.is_none()
        && players[0] != players[1]
        && prompt
            .args()
            .single::<String>()
            .map_or(true, |u| u != "casual");

    // create a prompt to see if the challenged person wants to play
    if ai_player_id.is_none() {
        let challenger = prompt.author.mention();
        let challengee_id = &players[0];
        let challengee = challengee_id.mention();

        let mode = match compete {
            true => "",
            false => "casual ",
        };

        let confirm = confirm_dialog(
            ctx,
            prompt,
            "Game Invite",
            &format!(
                "{}, you have been invited by {} to play a {} game of {}.
                To start the game, confirm this with a reaction within ten seconds.",
                challengee,
                challenger,
                mode,
                G::title()
            ),
            &challengee_id,
        )
        .await?;

        if !confirm {
            return Ok(());
        }
    }

    // create message and react
    let mut message = prompt
        .ereply(ctx, |e| e.title(G::title()).description("Loading game..."))
        .await?;
    for r in G::reactions() {
        message.react(ctx, r).await?;
    }

    let mut game_ctx = GameContext { players, turn: 0 };
    let mut time_left;

    macro_rules! forfeit {
        () => {
            time_left == 0.0
        };
    };

    macro_rules! update_field {
        () => {
            message
                .eedit(ctx, |e| {
                    e.title(G::title());
                    if let Some(desc) = cmd.options.desc {
                        e.description(desc);
                    }
                    e.field("Board", game.draw(&game_ctx), false);
                    let status = match game.status() {
                        _ if forfeit!() => {
                            format!("{} won by inactivity of {}.",
                                game_ctx.players[1-game_ctx.turn].mention(),
                                game_ctx.players[game_ctx.turn].mention(),
                            )
                        }
                        GameState::Win(p) => format!("{} won!", game_ctx.players[p].mention()),
                        GameState::Tie => String::from("It's a tie!"),
                        _ => format!(
                            "{}({}) plays next.\nTime left: {} seconds (updated every few seconds).",
                            game_ctx.players[game_ctx.turn].mention(),
                            G::figures()[game_ctx.turn],
                            time_left as u64
                        ),
                    };
                    e.field("Status", status, false)
                })
            .await?;
            };
    };

    'game: loop {
        let before_move = Instant::now();
        let play = loop {
            time_left = (timeout - before_move.elapsed().as_secs_f64()).max(0.0);

            update_field!();

            if forfeit!() {
                break 'game;
            }

            let is_ai_move = Some(game_ctx.turn) == ai_player_id;

            let idx = if is_ai_move {
                G::ai().unwrap().make_move(&game, ai_player_id.unwrap())
            } else {
                let reaction = message
                    .await_reaction(&ctx.shard)
                    .author_id(game_ctx.players[game_ctx.turn].id)
                    .removed(true)
                    .timeout(Duration::from_secs_f64(10.0))
                    .await;

                let reaction = tryc!(reaction);

                // if it is one of the given emojis, try to make that move
                tryc!(G::reactions()
                    .into_iter()
                    .position(|e| e == reaction.as_inner_ref().emoji))
            };

            let state = game.make_move(idx, game_ctx.turn);
            if state != GameState::Invalid {
                moves.push(idx as u8);
                break state;
            }

            if is_ai_move {
                // AI is the one that made the invalid move
                message
                    .ereply(ctx, |e| {
                        e.title("Programming error");
                        e.description(format!(
                            "The AI for this game sucks and tries to do invalid moves, {} pls fix.",
                            DISCORD_AUTHOR
                        ))
                    })
                    .await?;
                return Ok(());
            }
        };

        if play.is_finished() {
            break;
        } else {
            game_ctx.next_turn();
        }
    }

    update_field!();

    if compete {
        let winner = match game.status() {
            GameState::Win(p) => Some(p),
            _ if forfeit!() => Some(1 - game_ctx.turn),
            _ => None,
        };

        let server = *prompt.guild_id.ok_or("no server id")?.as_u64();
        let players = game_ctx
            .players
            .iter()
            .map(|p| *p.id.as_u64())
            .collect::<Vec<u64>>();

        log_game(game_name, server, &players, &moves, winner)?;
        elo::process_game(game_name, server, &players, winner)?;
    }

    Ok(())
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

lazy_static! {
    pub static ref NUMBERS: Vec<String> = (0..10)
        .map(|num| format!("{}\u{fe0f}\u{20e3}", num))
        .collect();
}

fn number_emoji(num: usize) -> ReactionType {
    ReactionType::Unicode(NUMBERS[num].clone())
}
