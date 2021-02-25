use crate::prelude::*;
use crate::ser::*;
use rand::prelude::*;
use rand::seq::SliceRandom;
use tracing::*;

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

#[command("trackme")]
#[description = "Definitely tracks your IP location from your discord username"]
async fn track(ctx: &Context, msg: &Message) -> CommandResult {
    let url = {
        let mut rng = thread_rng();
        let x = rng.gen::<f32>() * 40.0;
        let y = rng.gen::<f32>() * 40.0;
        format!("<https://www.google.com/maps/@{:.7}:{:.7},11z>", x, y)
    };
    msg.author.dm(ctx, |f| f.content(url)).await?;
    Ok(())
}

#[command]
#[description = "Displays info about a server"]
#[only_in(guilds)]
async fn serverinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(ctx).await.ok_or("no guild")?;
    let inline = true;
    let display_roles = false;

    let channels = &guild.channels(ctx).await?;

    msg.ereply(ctx, |e| {
        e.title(&guild.name);
        if let Some(desc) = &guild.description {
            e.description(desc);
        }
        if let Some(icon) = guild.icon_url() {
            e.thumbnail(icon);
        }
        if let Some(splash) = guild.splash_url() {
            e.image(splash);
        }

        e.field("Owner", guild.owner_id.mention(), inline);
        if let Some(url) = &guild.vanity_url_code {
            e.field("Url", url, inline);
        }
        e.field("Region", &guild.region, inline);

        if display_roles {
            let mut roles = guild.roles.values().collect::<Vec<_>>();
            roles.sort_by(|a, b| b.position.cmp(&a.position));
            let mut roles_string = String::new();
            let mut role_fields = 1;
            for role in roles.iter() {
                let mention = format!("{}\n", role.id.mention());
                if roles_string.len() + mention.len() > 1000 {
                    e.field(format!("Roles({})", role_fields), &roles_string, inline);
                    roles_string.clear();
                    role_fields += 1;
                }
                roles_string += &mention;
            }
            if roles_string.len() > 0 {
                let title = if role_fields == 1 {
                    "Roles".into()
                } else {
                    format!("Roles({})", role_fields)
                };
                e.field(title, roles_string, inline);
            }
        }

        e.field("Members", guild.member_count, inline);
        e.field("Boosts", guild.premium_subscription_count, inline);
        e.field("Emojis", guild.emojis.len(), inline);

        e.field(
            "Text Channels",
            channels
                .values()
                .filter(|c| c.kind == ChannelType::Text)
                .count(),
            inline,
        );
        e.field(
            "Voice Channels",
            channels
                .values()
                .filter(|c| c.kind == ChannelType::Voice)
                .count(),
            inline,
        );

        e
    })
    .await?;
    Ok(())
}

#[command]
pub async fn serveremojis(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(ctx).await.ok_or("no guild")?;
    let emojis = guild
        .emojis
        .iter()
        .map(|e| format!("{} ", e.1.mention()))
        .collect::<String>();

    msg.ereply(ctx, |e| {
        e.title("Emojis");
        e.description(&emojis)
    })
    .await?;

    event!(tracing::Level::INFO, "{}", emojis);
    Ok(())
}
