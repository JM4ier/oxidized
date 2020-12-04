use crate::prelude::*;
use crate::ShardManagerContainer;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
#[owners_only]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        msg.reply(ctx, "Shutting down!").await?;
        manager.lock().await.shutdown_all().await;
    } else {
        msg.reply(ctx, "There was a problem getting the shard manager")
            .await?;

        return Ok(());
    }

    Ok(())
}

#[command]
#[owners_only]
async fn repeat(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = msg.args();
    let count = args.single::<u32>()?;
    let word = args.single::<String>()?;
    for _ in 0..count {
        msg.channel_id.say(&ctx.http, &word).await?;
    }
    Ok(())
}
