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

    if let Channel::Guild(channel) = channel {
        let name = channel.name;
        if !name.contains("bot") && !name.contains("spam") {
            return Err(Reason::Unknown);
        }
    }
    Ok(())
}
