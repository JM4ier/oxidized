use crate::prelude::*;
use crate::ser::*;
use crate::ShardManagerContainer;
use std::fs::File;
use std::io::Read;
use std::sync::atomic::{AtomicU64, Ordering};

use tracing::*;

#[group]
#[owners_only]
#[prefix = "sudo"]
#[commands(quit, repeat, delete, debug, status, nick, cat, delet, stop_spam)]
pub struct Management;

#[command]
#[aliases(restart)]
#[description = "Stops the bot"]
async fn quit(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        msg.ereply(ctx, |e| {
            e.title("Shutting down!");
            e.description("Hopefully going alright")
        })
        .await?;
        manager.lock().await.shutdown_all().await;
    } else {
        msg.ereply(ctx, |e| {
            e.title("Error");
            e.description("There was a problem getting the shard manager")
        })
        .await?;
        return Ok(());
    }

    Ok(())
}

lazy_static! {
    static ref STOP_SPAM: AtomicU64 = AtomicU64::new(0);
}

#[command]
#[min_args(2)]
#[description = "Repeats a message n times."]
#[usage = "<n> <message>"]
#[example = "5 Hello, World!"]
async fn repeat(ctx: &Context, msg: &Message) -> CommandResult {
    let initial_spam = STOP_SPAM.load(Ordering::SeqCst);
    let mut args = msg.args();
    let count = args.single::<u32>()?;
    let word = args.rest();
    for _ in 0..count {
        msg.channel_id.say(&ctx.http, &word).await?;

        if initial_spam != STOP_SPAM.load(Ordering::SeqCst) {
            break;
        }
    }
    Ok(())
}

#[command("stopspam")]
async fn stop_spam(_: &Context, _: &Message) -> CommandResult {
    STOP_SPAM.fetch_add(1, Ordering::SeqCst);
    Ok(())
}

#[command]
#[description = "Deletes the bots messages. If no argument is provided, it goes through the last 100 messages and deletes the bots messages. If an argument 'x' is provided, it will go throught the last x messages of the channel."]
#[usage = "[<number>]"]
#[example = ""]
#[example = "25"]
async fn delete(ctx: &Context, msg: &Message) -> CommandResult {
    let delete_count = msg.args().single::<u64>().unwrap_or(100);
    let channel = msg.channel_id;

    let messages = channel
        .messages(ctx, |retriever| {
            retriever.before(msg.id).limit(delete_count)
        })
        .await?;

    for msg in messages.into_iter() {
        if msg.is_own(ctx).await {
            channel.delete_message(ctx, msg.id).await?;
        }
    }

    Ok(())
}

#[command]
#[description = "Prints the messages content to the console."]
async fn debug(_: &Context, msg: &Message) -> CommandResult {
    event!(tracing::Level::INFO, "{}", msg.content);
    Ok(())
}

#[command]
#[min_args(1)]
#[description = "Sets the activity displayed under the bots name. The first argument needs to be either 'playing', 'listening', 'competing' or 'streaming'. In the case of the first three, it takes the rest of the passed arguments as displayed game/music/competion. In case of streaming it interprets the second argument as the stream URL and the rest as stream name."]
#[usage = "(playing | listening | competing | streaming <url>) <activity>"]
#[example = "playing Factorio"]
#[example = "streaming https://www.twitch.tv/badplayzrl Rocket League"]
#[example = "listening your commands"]
async fn status(ctx: &Context, msg: &Message) -> CommandResult {
    let mut args = msg.args();
    let activity = match args.single::<String>()?.as_str() {
        "playing" => Activity::playing(args.rest()),
        "listening" => Activity::listening(args.rest()),
        "competing" => Activity::competing(args.rest()),
        "streaming" => {
            let url = args.single::<String>()?;
            Activity::streaming(args.rest(), &url)
        }
        _ => return Err(From::from("invalid activity type")),
    };
    ctx.shard.set_activity(Some(activity));
    Ok(())
}

#[command]
#[description = "Changes the nickname of the bot for the current guild"]
#[usage = "[<nickname>]"]
#[example = ""]
#[example = "bot"]
async fn nick(ctx: &Context, msg: &Message) -> CommandResult {
    let nick = msg.args().single::<String>().ok();
    let nick = nick.as_ref().map(String::as_str);
    let guild = *msg
        .guild_id
        .ok_or("This message was not send in a guild.")?
        .as_u64();
    ctx.http.edit_nickname(guild, nick).await?;
    Ok(())
}

#[command]
#[description = "Reads a file"]
#[usage = "<file>"]
#[min_args(1)]
async fn cat(ctx: &Context, msg: &Message) -> CommandResult {
    let args = msg.args();
    let file_name = args.rest();
    let mut file = File::open(file_name)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    msg.reply(ctx, content).await?;
    Ok(())
}

#[command]
#[description = "deletes a message"]
async fn delet(ctx: &Context, msg: &Message) -> CommandResult {
    let to_delet = msg
        .message_reference
        .as_ref()
        .and_then(|m| m.message_id)
        .ok_or("no reference to earlier message")?;

    msg.delete(ctx).await?;

    let around_msgs = msg
        .channel_id
        .messages(ctx, |r| r.around(to_delet).limit(1))
        .await?;

    for msg in around_msgs {
        if msg.id == to_delet {
            msg.delete(ctx).await?;
        }
    }

    Ok(())
}
