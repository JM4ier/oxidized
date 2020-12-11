use crate::tryc;
use serenity::builder::*;
use serenity::framework::standard::{macros::command, macros::*, CommandResult};
use serenity::model::{channel::*, user::*};
use serenity::prelude::*;

mod tictactoe;
mod ultimate;

#[group]
#[prefix = "play"]
#[commands(tictactoe, ultimate)]
pub struct PlayGroup;

#[command]
#[only_in(guilds)]
#[description("The classic 3x3 game without strategy.")]
async fn tictactoe(ctx: &Context, prompt: &Message) -> CommandResult {
    pvp_game(ctx, prompt, tictactoe::TTTField::default()).await
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
    pvp_game(ctx, prompt, ultimate::UltimateGame::new()).await
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

/// All functions a game must possess
trait PvpGame {
    /// All possible reactions to this game
    fn reactions() -> Vec<ReactionType>;
    /// Display the current board in the discord message
    fn edit_message<'e>(&self, m: &'e mut EditMessage, ctx: &GameContext) -> &'e mut EditMessage;
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

fn number_emoji(num: usize) -> ReactionType {
    ReactionType::Unicode(format!("{}\u{fe0f}\u{20e3}", num))
}
