use crate::prelude::*;
use crate::ShardManagerContainer;
use serenity::{
    framework::standard::{
        macros::{command, *},
        *,
    },
    futures::*,
    model::prelude::*,
    prelude::*,
};

use tracing::*;

#[group]
#[owners_only]
#[prefix = "sudo"]
#[commands(quit, repeat, delete, debug)]
pub struct Management;

#[command]
#[aliases(restart)]
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
async fn repeat(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = msg.args();
    let count = args.single::<u32>()?;
    let word = args.rest();
    for _ in 0..count {
        msg.channel_id.say(&ctx.http, &word).await?;
    }
    Ok(())
}

#[command]
async fn delete(ctx: &Context, msg: &Message) -> CommandResult {
    let mut delete_count = msg.args().single::<i64>().unwrap_or(100);
    let mut messages = msg.channel_id.messages_iter(&ctx).boxed();
    while let Some(message_res) = messages.next().await {
        if let Ok(msg) = message_res {
            if msg.is_own(&ctx.cache).await {
                let _ = msg.delete(&ctx.http).await;
            }
        }

        delete_count -= 1;
        if delete_count <= 0 {
            break;
        }
    }

    Ok(())
}

#[command]
async fn debug(_: &Context, msg: &Message) -> CommandResult {
    event!(tracing::Level::INFO, "{}", msg.content);
    Ok(())
}
