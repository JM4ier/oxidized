#![allow(unused)]
use serenity::framework::standard::*;
use serenity::model::prelude::*;

pub const NAME: &'static str = env!("CARGO_PKG_NAME");
pub const AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const DISCORD_AUTHOR: &'static str = "<@!177498563637542921>";

pub trait MessageArgs {
    fn args(&self) -> Args;
}
impl MessageArgs for Message {
    fn args(&self) -> Args {
        let delimiter = [Delimiter::Single(' ')];
        let mut args = Args::new(&self.content, &delimiter);
        let _ = args.single::<String>();
        Args::new(args.rest(), &delimiter)
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
