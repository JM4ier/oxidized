#![allow(unused)]
use super::command_groups;
use crate::ser::*;
use async_trait::async_trait;
use chrono::prelude::*;
use rusqlite::{params, Connection};
use std::collections::*;

pub use crate::util::*;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BUILD_DATE: &str = env!("BUILD_DATE");
pub const DISCORD_AUTHOR: &str = "<@!177498563637542921>";

lazy_static! {
    pub static ref TEST_INSTANCE: bool = std::env::args().any(|arg| arg == "-t" || arg == "--test");
    pub static ref PREFIX: &'static str = if *TEST_INSTANCE { "==" } else { "=" };
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

pub fn commands() -> Vec<&'static Command> {
    let mut groups = VecDeque::from(command_groups());
    let mut cmds = Vec::new();

    while let Some(group) = groups.pop_front() {
        for c in group.options.commands {
            cmds.push(c.clone());
        }
        for g in group.options.sub_groups.iter() {
            groups.push_back(g);
        }
    }
    cmds
}

fn command_names() -> Vec<&'static str> {
    commands()
        .iter()
        .flat_map(|c| c.options.names)
        .map(|&c| c)
        .collect()
}

pub trait MessageArgs {
    fn args(&self) -> Args;
}
impl MessageArgs for Message {
    fn args(&self) -> Args {
        let delimiter = [Delimiter::Single(' ')];

        // remove leading prefix
        let content = self.content.clone().split_off(PREFIX.len());

        let mut args = Args::new(&content, &delimiter);
        let cmds = command_names();

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

/// concatenates a list of strings into larger strings, while respecting the maximum length of an
/// embed field.
///
/// Note: Strings must not be longer than 2000 characters.
pub fn split_into_fields(parts: &[String], default: &str) -> Vec<String> {
    const LIMIT: usize = 2000;
    let mut fields = Vec::new();
    let mut field = String::new();
    for part in parts.iter() {
        if field.len() + part.len() > LIMIT {
            fields.push(field);
            field = String::new();
        }
        field += part;
    }
    if field.len() > 0 {
        fields.push(field);
    }
    if fields.len() == 0 {
        fields.push(default.into());
    }
    fields
}

fn embed_template<'u, 'c: 'u>(msg: &'u Message, e: &'c mut CreateEmbed) -> &'c mut CreateEmbed {
    let author = &msg.author;

    let name = msg
        .member
        .as_ref()
        .and_then(|m| m.nick.as_ref())
        .unwrap_or(&author.name);

    e.footer(|f| {
        author.avatar_url().map(|url| f.icon_url(url));
        f.text(format!("summoned by {}", name))
    });

    e.color(Color::new(0x046B2F))
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
                    embed_template(self, e);
                    fun(e)
                })
            })
            .await
    }
}
#[async_trait]
pub trait EmbedEdit {
    async fn eedit<F>(&mut self, ctx: &Context, f: F) -> Result<(), SerenityError>
    where
        F: Send + FnOnce(&mut CreateEmbed) -> &mut CreateEmbed;
}
#[async_trait]
impl EmbedEdit for Message {
    async fn eedit<F>(&mut self, ctx: &Context, fun: F) -> Result<(), SerenityError>
    where
        F: Send + FnOnce(&mut CreateEmbed) -> &mut CreateEmbed,
    {
        let footer = self.embeds.iter().filter_map(|e| e.footer.clone()).next();
        let colour = self.embeds.iter().next().map(|e| e.colour);

        self.edit(ctx, |m| {
            m.embed(|e| {
                colour.map(|c| e.colour(c));
                footer.map(|footer| {
                    e.footer(|f| {
                        footer.icon_url.map(|i| f.icon_url(i));
                        f.text(footer.text)
                    })
                });
                fun(e)
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
