use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::*;
use serenity::{futures::*, prelude::*};

#[command]
#[help_available]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;
    Ok(())
}

#[command]
#[owners_only]
async fn elthision(ctx: &Context, msg: &Message) -> CommandResult {
    // delete all sent messages within the last 5000 sent messages
    let mut i = 0;
    let mut messages = msg.channel_id.messages_iter(&ctx).boxed();
    while let Some(message_res) = messages.next().await {
        if let Ok(msg) = message_res {
            if msg.is_own(&ctx.cache).await {
                let _ = msg.delete(&ctx.http).await;
            }
        }
        i += 1;
        if i > 100 {
            break;
        }
    }

    Ok(())
}
