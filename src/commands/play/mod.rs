use crate::{prelude::*, tryc};
use lazy_static::*;
use serenity::framework::standard::{macros::command, macros::*, CommandResult, *};
use serenity::model::{channel::*, user::*};
use serenity::prelude::*;
use serenity::utils::Color;

mod mcts;
mod minimax;
mod random_ai;
mod tictactoe;
mod ultimate;
use mcts::*;
use minimax::*;
use random_ai::*;

#[group]
#[help_available]
#[prefix = "play"]
#[commands(tictactoe, ultimate)]
pub struct Games;

#[command]
#[only_in(guilds)]
#[description("The classic 3x3 game without strategy.")]
#[usage = "<enemy_player>"]
async fn tictactoe(ctx: &Context, prompt: &Message) -> CommandResult {
    pvp_game(ctx, prompt, tictactoe::TTTField::default(), "tictactoe").await
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
async fn ultimate(ctx: &Context, prompt: &Message) -> CommandResult {
    pvp_game(ctx, prompt, ultimate::UltimateGame::new(), "ultimate").await
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
    cmd: &str,
) -> CommandResult {
    let cmds = commands();
    let cmd = cmds
        .iter()
        .filter(|c| c.options.names.contains(&cmd))
        .next()
        .unwrap();

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

    // create message and react
    let mut message = prompt
        .ereply(ctx, |e| e.title(G::title()).description("Loading game..."))
        .await?;
    for r in G::reactions() {
        message.react(&ctx.http, r).await?;
    }

    let mut game_ctx = GameContext { players, turn: 0 };

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
                        GameState::Win(p) => format!("{} won!", game_ctx.players[p].mention()),
                        GameState::Tie => String::from("It's a tie!"),
                        _ => format!(
                            "{}({}) plays next.",
                            game_ctx.players[game_ctx.turn].mention(),
                            G::figures()[game_ctx.turn]
                        ),
                    };
                    e.field("Status", status, false)
                })
                .await?;
        };
    };

    loop {
        update_field!();
        let play = loop {
            let is_ai_move = Some(game_ctx.turn) == ai_player_id;

            let idx = if is_ai_move {
                G::ai().unwrap().make_move(&game, ai_player_id.unwrap())
            } else {
                let reaction = message.await_reaction(&ctx.shard).await;
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
