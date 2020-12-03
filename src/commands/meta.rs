use crate::prelude::*;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::*;
use serenity::{futures::*, prelude::*};
use std::time::*;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let before_ping = Instant::now();
    let mut ping = msg
        .channel_id
        .send_message(&ctx.http, |m| m.embed(|e| e.title("Ping Stats")))
        .await?;
    let elapsed = before_ping.elapsed().as_millis();
    ping.edit(&ctx.http, |m| {
        m.embed(|e| {
            e.title("Ping Stats");
            e.field("Roundtrip", format!("`{}ms`", elapsed), false)
        })
    })
    .await?;
    Ok(())
}

#[command]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("{} info page", NAME));
                e.description("All the info you could want");
                e.field(
                    "Author",
                    format!(" {} | <https://github.com/JM4ier>", DISCORD_AUTHOR),
                    false,
                );
                e.field("Version", format!("{} v{}", NAME, VERSION), false);
                e.field("Source", "<https://github.com/JM4ier/oxidized>", false)
            })
        })
        .await?;
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
