use crate::ser::*;

#[check]
#[name = "Spam"]
pub async fn only_in_spam(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> Result<(), Reason> {
    let channel = msg
        .channel_id
        .to_channel(ctx)
        .await
        .map_err(|_| Reason::Unknown)?;

    match channel {
        Channel::Guild(channel) => {
            let name = channel.name;
            if name.contains("bot") || name.contains("spam") {
                Ok(())
            } else {
                Err(Reason::Unknown)
            }
        }
        _ => Ok(()),
    }
}
