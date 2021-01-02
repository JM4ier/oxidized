use crate::prelude::*;
use rand::seq::SliceRandom;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::*;

#[command]
#[min_args(1)]
#[description = "Chooses a random element out of a number of arguments"]
#[example = r#""play Factorio" "improve the bot""#]
async fn random(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = msg.args();
    let mut words = Vec::new();
    while let Ok(word) = args.single::<String>() {
        words.push(word)
    }

    let word = words.choose(&mut rand::thread_rng());
    msg.ereply(ctx, |e| {
        e.title("Random element");
        if let Some(word) = word {
            e.description(format!("{} has been chosen.", word))
        } else {
            e.colour(Colour::RED);
            e.description("Please specify a list of words.")
        }
    })
    .await?;

    Ok(())
}
