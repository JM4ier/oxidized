use crate::{prelude::*, tryc};
use lazy_static::*;
use rusqlite::{params, Result};
use serenity::framework::standard::{macros::command, macros::*, CommandResult};
use serenity::model::{channel::*, id::*, user::*};
use serenity::prelude::*;
use serenity::utils::Color;
use std::time::*;

mod mcts;
mod minimax;
mod random_ai;
mod tictactoe;
mod ultimate;
use minimax::*;
use random_ai::*;

#[group]
#[help_available]
#[prefix = "play"]
#[commands(tictactoe, ultimate, leaderboard)]
pub struct Games;

#[command]
#[only_in(guilds)]
#[description("The classic 3x3 game without strategy.")]
#[usage = "<enemy_player>"]
#[bucket("game")]
async fn tictactoe(ctx: &Context, prompt: &Message) -> CommandResult {
    pvp_game(
        ctx,
        prompt,
        tictactoe::TTTField::default(),
        "tictactoe",
        60.0,
    )
    .await
}

#[command]
#[only_in(guilds)]
#[description(
    "You play on a 3x3 grid of tictactoe fields.
Where you move in the small field determines which field your opponent is going to play in next.
If that targeted field is already occupied (won/lost/tied), the field with the next bigger index is chosen.
A win in a small field counts as a mark on the big field.
You win if you have three in a row, column or diagonal in the big field."
)]
#[usage = "<enemy_player>"]
#[bucket("game")]
async fn ultimate(ctx: &Context, prompt: &Message) -> CommandResult {
    pvp_game(ctx, prompt, ultimate::UltimateGame::new(), "ultimate", 60.0).await
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
    fn ai() -> Option<Box<dyn AiPlayer<Self>>> {
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

    let compete = ai_player_id.is_none()
        && players[0] != players[1]
        && prompt
            .args()
            .single::<String>()
            .map_or(true, |u| u != "casual");

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

    if ai_player_id.is_none() {
        let challenger = prompt.author.mention();
        let challengee_id = &players[0];
        let challengee = challengee_id.mention();

        let mode = match compete {
            true => "",
            false => "casual ",
        };

        let confirm = prompt
            .ereply(ctx, |e| {
                e.title("Game Invite");
                e.description(format!(
                    "{}, you have been invited by {} to play a {} game of {}.\nTo start the game, confirm this with a reaction within ten seconds.",
                    challengee, challenger, mode, game_name
                ))
            })
            .await?;
        confirm
            .react(ctx, ReactionType::Unicode(String::from("⬆️")))
            .await?;

        let begin = Instant::now();

        let confirmed = loop {
            let elapsed = begin.elapsed().as_secs_f64();
            if elapsed >= 10.0 {
                break false;
            }
            let reaction = confirm
                .await_reaction(ctx)
                .timeout(Duration::from_secs_f64(10.0 - elapsed))
                .await;
            let reaction = tryc!(reaction);
            let reaction = reaction.as_inner_ref();
            if reaction.user_id == Some(challengee_id.id) {
                break true;
            }
        };

        confirm.delete(ctx).await?;

        if !confirmed {
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
            time_left = timeout - before_move.elapsed().as_secs_f64();
            time_left = time_left.max(0.0);

            update_field!();

            let is_ai_move = Some(game_ctx.turn) == ai_player_id;

            if forfeit!() {
                break 'game;
            }

            let idx = if is_ai_move {
                G::ai().unwrap().make_move(&game, ai_player_id.unwrap())
            } else {
                let reaction = message
                    .await_reaction(&ctx.shard)
                    .timeout(Duration::from_secs_f64(10.0))
                    .await;

                let reaction = tryc!(reaction);
                let reaction = reaction.as_inner_ref();

                // check player who has reacted
                if Some(game_ctx.players[game_ctx.turn].id) != reaction.user_id {
                    continue;
                }

                // if it is one of the given emojis, try to make that move
                tryc!(G::reactions().into_iter().position(|e| e == reaction.emoji))
            };

            let state = game.make_move(idx, game_ctx.turn);
            if state != GameState::Invalid {
                moves.push(idx as u8);
                break state;
            }

            // ai made an invalid move
            if is_ai_move {
                let reply = format!(
                    "The AI for this game sucks and tries to do invalid moves, {} pls fix.",
                    DISCORD_AUTHOR
                );
                message
                    .ereply(ctx, |e| e.title("Programming error").description(reply))
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
        let server = *prompt.guild_id.ok_or("no server id")?.as_u64();
        let mut players = Vec::new();
        let mut elo = Vec::new();
        for p in 0..2 {
            players.push(*game_ctx.players[p].id.as_u64());
            elo.push(get_elo(server, players[p], game_name)?);
        }

        let result = match game.status() {
            GameState::Win(p) => (p as u8) + 1,
            _ if forfeit!() => 2 - game_ctx.turn as u8,
            _ => 0,
        };

        log_game(server, players[0], players[1], game_name, moves, result)?;

        // expected score for player 0
        let exp0 = 1.0 / (1.0 + 10.0_f64.powf((elo[1] - elo[0]) / 400.0));

        // actual score for player 0
        let score0 = match game.status() {
            GameState::Win(p) => 1.0 - p as f64,
            _ if forfeit!() => game_ctx.turn as f64,
            _ => 0.5,
        };

        const K: f64 = 40.0;

        // calculate elo addition/subtraction and clamp
        let d_elo = K * (score0 - exp0);

        // update elo
        set_elo(server, players[0], game_name, elo[0] + d_elo)?;
        set_elo(server, players[1], game_name, elo[1] - d_elo)?;
    }

    Ok(())
}

#[command]
async fn leaderboard(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = msg.args();
    let game = args.single::<String>()?;

    if game.chars().any(|c| !c.is_ascii_alphabetic()) {
        Err("sqli")?;
    }

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

    let mut leaderboard = String::new();

    for (idx, (user, elo)) in players.into_iter().enumerate() {
        let idx = idx + 1;
        let suffix = match idx % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        };

        let rank = format!("{: >3}{}", idx, suffix);
        let points = format!("{:0>4}", elo as i64);

        let user = match user.to_user(ctx).await {
            Ok(user) => user.mention().to_string(),
            Err(_) => String::from("<invalid user>"),
        };

        let lb_entry = format!("\u{200b}{} {} {}\n", rank, points, user);

        if leaderboard.len() + lb_entry.len() > 2000 {
            break;
        }
        leaderboard += &lb_entry;
    }

    if leaderboard.len() == 0 {
        leaderboard += "This leaderboard is empty.";
    }

    msg.ereply(ctx, |e| {
        e.title(format!("{} Leaderboard", game));
        e.description(leaderboard)
    })
    .await?;

    Ok(())
}

type Elo = f64;

fn get_elo(server: u64, player: u64, game: &str) -> Result<Elo> {
    let player = format!("{}", player);
    let server = format!("{}", server);
    let db = db()?;
    let elo: Elo = db
        .query_row(
            &format!(
                "SELECT elo FROM {} WHERE player = ?2 AND server = ?1",
                elo_table(game)
            ),
            params!(server, player),
            |row| row.get(0),
        )
        .unwrap_or(1200.0);
    Ok(elo)
}

fn set_elo(server: u64, player: u64, game: &str, elo: Elo) -> Result<()> {
    let player = format!("{}", player);
    let server = format!("{}", server);

    let db = db()?;
    let affected = db.execute(
        &format!(
            "UPDATE {} SET elo = ?3 WHERE player=?1 AND server=?2;",
            elo_table(game)
        ),
        params!(player, server, elo),
    )?;

    if affected == 0 {
        db.execute(
            &format!(
                "INSERT INTO {} (server, player, elo) VALUES (?1, ?2, ?3);",
                elo_table(game)
            ),
            params!(server, player, elo),
        )?;
    }
    Ok(())
}

fn log_game(
    server: u64,
    player1: u64,
    player2: u64,
    game: &str,
    moves: Vec<u8>,
    result: u8,
) -> Result<()> {
    let player1 = format!("{}", player1);
    let player2 = format!("{}", player2);
    let server = format!("{}", server);
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
