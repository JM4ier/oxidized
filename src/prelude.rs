#![allow(unused)]
use super::command_groups;
use rusqlite::{params, Connection};
use serenity::framework::standard::*;
use serenity::model::prelude::*;
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
