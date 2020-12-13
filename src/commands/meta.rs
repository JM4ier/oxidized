use crate::prelude::*;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::channel::*;
use serenity::prelude::*;
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
#[aliases(about)]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("{} info page", NAME));
                e.description("All the info you could want");
                e.field(
                    "Author",
                    format!(
                        " {} | [GitHub Profile](https://github.com/JM4ier)",
                        DISCORD_AUTHOR
                    ),
                    false,
                );
                e.field("Version", format!("{} v{}", NAME, VERSION), false);
                e.field(
                    "Source",
                    "[GitHub Repository](https://github.com/JM4ier/oxidized)",
                    false,
                );
                e.field("Build", format!("`{}`", BUILD_DATE), false)
            })
        })
        .await?;
    Ok(())
}
