use crate::config::Config;
use serenity::{
    all::*,
    builder::{CreateEmbed, CreateMessage},
    prelude::SerenityError,
};

pub async fn log_ticket_action(
    ctx: &Context,
    action: &str,
    user: &User,
    channel: &GuildChannel,
) -> Result<(), SerenityError> {
    let config = Config::get();
    let log_channel = ChannelId::new(config.log_channel_id);

    let embed = CreateEmbed::new()
        .title(format!("Ticket {}", action))
        .field("User", user.name.clone(), true)
        .field("Channel", channel.name.clone(), true)
        .field("Action", action, false)
        .timestamp(Timestamp::now())
        .color(match action {
            "Opened" => 0x00ff00,
            "Closed" => 0xff0000,
            "User Added" => 0x0000ff,
            "User Removed" => 0xff00ff,
            _ => 0xffa500,
        });

    log_channel
        .send_message(&ctx.http, CreateMessage::new().embed(embed))
        .await?;

    Ok(())
}
