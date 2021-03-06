#[macro_use]
extern crate lazy_static;

mod commands;
mod prelude;
mod ser;
mod util;

use crate::ser::*;
use commands::{brainfuck::*, easteregg::*, meta::*, owner::*, util::*};
use prelude::*;
use std::{collections::HashSet, env, sync::Arc};
use tracing::*;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        ctx.shard.set_activity(Some(Activity::listening("=help")));
    }

    async fn resume(&self, _: Context, _: ResumedEvent) {
        info!("Resumed");
    }
}

#[help]
#[command_not_found_text = "Could not find `{}`."]
#[max_levenshtein_distance(3)]
#[lacking_permissions = "Hide"]
#[wrong_channel = "Strike"]
async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
pub async fn on_dispatch_error(ctx: &Context, msg: &Message, _: DispatchError) {
    let clown = ReactionType::Unicode("🤡".into());
    msg.react(ctx, clown).await.ok();
}

#[hook]
pub async fn before(_ctx: &Context, msg: &Message, _cmd: &str) -> bool {
    *msg.channel_id.as_u64() != 819966095070330950
}

#[hook]
pub async fn after(ctx: &Context, msg: &Message, _: &str, err: Result<(), CommandError>) {
    let reaction = match err {
        Err(err) => {
            event!(
                tracing::Level::INFO,
                r#"Message "{}" caused "{}""#,
                msg.content,
                err
            );
            "🤦"
        }
        Ok(_) => "👌",
    };
    let reaction = ReactionType::Unicode(reaction.into());
    msg.react(ctx, reaction).await.ok();
}

#[group]
#[help_available]
#[commands(bug, info, ping, solution, random, track, serverinfo, serveremojis)]
struct General;

pub fn command_groups() -> Vec<&'static CommandGroup> {
    use crate::commands::play::*;
    vec![
        &GENERAL_GROUP,
        &MANAGEMENT_GROUP,
        &GAMES_GROUP,
        &LEADERBOARD_GROUP,
        &BRAINFUCK_GROUP,
    ]
}

#[tokio::main]
async fn main() {
    // Initialize the logger to use environment variables.
    //
    // In this case, a good default is setting the environment variable
    // `RUST_LOG` to debug`.
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to start the logger");

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let http = Http::new_with_token(&token);

    // We will fetch your bot's owners and id
    let (owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    // Create the framework
    let mut framework = StandardFramework::new()
        .help(&HELP)
        .configure(|c| c.owners(owners).prefix(&PREFIX))
        .on_dispatch_error(on_dispatch_error)
        .before(before)
        .after(after)
        .bucket("brainfuck", |b| b.time_span(10).limit(5))
        .await
        .bucket("game", |b| b.time_span(60).limit(6))
        .await;

    for group in command_groups() {
        framework = framework.group(group);
    }

    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    event!(tracing::Level::INFO, "Started bot at {}.", *START_DATE);

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
