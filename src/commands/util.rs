use crate::prelude::*;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
async fn random(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = msg.args();

    let mut words = Vec::new();
    while let Ok(word) = args.single::<String>() {
        words.push(word);
    }

    if words.len() > 0 {
        let word = &words[rand::random::<usize>() % words.len()];
        msg.channel_id.say(&ctx.http, word).await?;
    }

    Ok(())
}
