use crate::prelude::EmbedReply;
use crate::ser::*;
use crate::tryc;
use std::time::*;

const TIMEOUT: f64 = 20.0;

pub async fn confirm_dialog(
    ctx: &Context,
    msg: &Message,
    title: &str,
    body: &str,
    person: &User,
) -> CommandResult<bool> {
    // create dialog with title and body text
    let dialog = msg
        .ereply(ctx, |e| e.title(title).description(body))
        .await?;

    // add one reaction for an easy reply
    dialog
        .react(ctx, ReactionType::Unicode(String::from("⬆️")))
        .await?;

    // wait until there is a reaction by the correct person
    let begin = Instant::now();
    let confirmed = loop {
        let elapsed = begin.elapsed().as_secs_f64();
        if elapsed >= TIMEOUT {
            break false;
        }
        let reaction = dialog
            .await_reaction(ctx)
            .timeout(Duration::from_secs_f64(TIMEOUT - elapsed))
            .await;
        let reaction = tryc!(reaction);
        let reaction = reaction.as_inner_ref();
        if reaction.user_id == Some(person.id) {
            break true;
        }
    };

    // clean up dialog afterwards
    dialog.delete(ctx).await?;

    Ok(confirmed)
}
