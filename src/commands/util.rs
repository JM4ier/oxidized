use crate::prelude::*;
use rand::seq::SliceRandom;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::*;

#[command]
async fn random(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = msg.args();
    let mut words = Vec::new();
    while let Ok(word) = args.single::<String>() {
        words.push(word)
    }

    let word = words.choose(&mut rand::thread_rng());
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("Random element");
                if let Some(word) = word {
                    e.description(format!("{} has chosen `{}` for you", NAME, word))
                } else {
                    e.colour(Colour::RED);
                    e.description("Please specify a list of words.")
                }
            })
        })
        .await?;

    Ok(())
}
