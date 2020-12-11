mod commands;
mod prelude;

use serenity::{
    async_trait,
    client::bridge::gateway::ShardManager,
    framework::{standard::macros::*, standard::*, StandardFramework},
    http::Http,
    model::{channel::*, event::ResumedEvent, gateway::Ready, id::*},
    prelude::*,
};
use std::{collections::HashSet, env, sync::Arc};

use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use commands::{easteregg::*, meta::*, owner::*, play::*, util::*};

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
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
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
pub async fn on_dispatch_error(ctx: &Context, msg: &Message, err: DispatchError) {
    if let DispatchError::OnlyForOwners = err {
        let err_msg = format!(
            "{} is not in the sudoers file.\nThis incident will be reported.",
            msg.author.mention()
        );
        let _ = msg.channel_id.say(&ctx.http, err_msg).await;
    }
    println!("{:?}", err);
}

#[group]
#[help_available]
#[commands(info, ping, solution, random)]
struct General;

pub fn command_groups() -> Vec<&'static CommandGroup> {
    vec![&GENERAL_GROUP, &MANAGEMENT_GROUP, &GAMES_GROUP]
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
        .configure(|c| c.owners(owners).prefix("="))
        .on_dispatch_error(on_dispatch_error);

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

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
