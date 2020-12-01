use crate::prelude::*;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
async fn solution(ctx: &Context, msg: &Message) -> CommandResult {
    let bad_answers = vec![
        "You don't have permission to run this command!",
        "Didn't I already tell you that you lack permissions?",
        "I'm not giving you the solutions, dude.",
        "No solutions for you, for the last time",
        "So you really want to know the solutions?",
        "Wouldn't you like to know, weatherboy",
        "Do you really think a bot would give you the solutions?",
        "No.",
    ];
    let answers = vec![
        "An internal error has occured while displaying the solutions.",
        "The solutions for this week have not been uploaded yet.",
        "You need to enter the correct 16 digit pin number:",
        "You need to enter some 16 digit pin number:",
    ];
    let num_answers = vec![
        "Do you also know the three digits on the back?",
        "Ugh fine, here you go: || It is trivial. ||",
        "Here you go: <https://bit.ly/3fOGIuc>",
        "```python\nfrom eprog_solutions import *\n```",
        "You have not specified which week, try again.",
    ];

    use rand::*;
    use rand_pcg::*;

    let mut rng = Pcg64::seed_from_u64(*msg.author.id.as_u64());

    let answer_pool;

    if rng.gen() {
        answer_pool = bad_answers;
    } else if msg.args().rest().len() > 0 {
        answer_pool = num_answers;
    } else {
        answer_pool = answers;
    }

    let answer = answer_pool[rand::random::<usize>() % answer_pool.len()];

    msg.channel_id.say(&ctx.http, answer).await?;
    Ok(())
}
