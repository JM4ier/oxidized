use serenity::framework::standard::*;
use serenity::model::prelude::*;

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
