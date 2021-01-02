#![allow(unused)]
use super::command_groups;
use async_trait::async_trait;
use chrono::prelude::*;
use rusqlite::{params, Connection};
use serenity::builder::*;
use serenity::framework::standard::*;
use serenity::model::prelude::Message;
use serenity::prelude::*;
use std::collections::*;

pub const NAME: &'static str = env!("CARGO_PKG_NAME");
pub const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const BUILD_DATE: &'static str = env!("BUILD_DATE");
pub const DISCORD_AUTHOR: &'static str = "<@!177498563637542921>";
pub const PREFIX: &'static str = "=";

fn commands() -> Vec<&'static str> {
    let mut groups = VecDeque::from(command_groups());
    let mut cmds = Vec::new();

    while let Some(group) = groups.pop_front() {
        for c in group.options.commands {
            for n in c.options.names {
                cmds.push(*n);
            }
        }
        for g in group.options.sub_groups.iter() {
            groups.push_back(g);
        }
    }
    cmds
}

pub trait MessageArgs {
    fn args(&self) -> Args;
}
impl MessageArgs for Message {
    fn args(&self) -> Args {
        let delimiter = [Delimiter::Single(' ')];

        // remove leading prefix
        let content = self.content.split(PREFIX).collect::<String>();

        let mut args = Args::new(&content, &delimiter);
        let cmds = commands();

        // remove all group prefixes to find command
        loop {
            match args.single::<String>() {
                Ok(arg) => {
                    if cmds.contains(&&arg[..]) {
                        break;
                    }
                }
                Err(_) => return Args::new("", &delimiter),
            }
        }

        let mut args = Args::new(args.rest(), &delimiter);
        args.quoted();
        args
    }
}

#[async_trait]
pub trait EmbedReply {
    async fn ereply<F>(&self, ctx: &Context, f: F) -> Result<Message, SerenityError>
    where
        F: Send + FnOnce(&mut CreateEmbed) -> &mut CreateEmbed;
}
#[async_trait]
impl EmbedReply for Message {
    async fn ereply<F>(&self, ctx: &Context, fun: F) -> Result<Message, SerenityError>
    where
        F: Send + FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        self.channel_id
            .send_message(ctx, |m| {
                m.embed(|e| {
                    fun(e);
                    e.footer(|f| {
                        f.text(format!(
                            "on behalf of {}#{:04}",
                            self.author.name, self.author.discriminator
                        ));
                        if let Some(avatar) = self.author.avatar_url() {
                            f.icon_url(avatar);
                        }
                        f
                    })
                })
            })
            .await
    }
}

// Tries to Pattern match an option
//
// If it fails, it continues in the next loop iteration
#[macro_export]
macro_rules! tryc {
    ($maybe:expr) => {
        if let Some(e) = $maybe {
            e
        } else {
            continue;
        }
    };
}

pub fn db() -> rusqlite::Result<Connection> {
    Connection::open("./oxidized.db")
}

lazy_static! {
    pub static ref START_DATE: String = Utc::now().format("UTC %Y-%m-%d %H:%M:%S").to_string();
    pub static ref SYSTEM_NAME: String = {
        std::process::Command::new("sh")
            .arg("-c")
            .arg("uname -n")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or(String::from("unknown"))
    };
}
