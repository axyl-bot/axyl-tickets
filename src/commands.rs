use crate::config::Config;
use serenity::{
    all::*,
    builder::{CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, CreateMessage},
};

pub async fn init(ctx: &Context, command: &CommandInteraction) -> String {
    let config = Config::get();
    let embed = CreateEmbed::new()
        .title("Support Ticket")
        .description("Click the button below to open a new support ticket.")
        .color(0x00ff00)
        .footer(CreateEmbedFooter::new(format!(
            "Ticket Category ID: {}",
            config.category_id
        )));

    let button = CreateButton::new("open_ticket")
        .label("Open Ticket")
        .style(ButtonStyle::Primary);

    let action_row = CreateActionRow::Buttons(vec![button]);

    if let Err(why) = command
        .channel_id
        .send_message(
            &ctx.http,
            CreateMessage::new()
                .embed(embed)
                .components(vec![action_row]),
        )
        .await
    {
        format!("Failed to send ticket embed: {}", why)
    } else {
        "Ticket system initialized successfully.".to_string()
    }
}

pub async fn close(ctx: &Context, command: &CommandInteraction) -> String {
    let config = Config::get();
    let embed = CreateEmbed::new()
        .title("Closing Ticket")
        .description("This ticket will be closed in 5 seconds. Click the button below to cancel.")
        .color(0xff0000)
        .footer(CreateEmbedFooter::new(format!(
            "Ticket Category ID: {}",
            config.category_id
        )));

    let button = CreateButton::new("cancel_close")
        .label("Cancel")
        .style(ButtonStyle::Danger);

    let action_row = CreateActionRow::Buttons(vec![button]);

    if let Err(why) = command
        .channel_id
        .send_message(
            &ctx.http,
            CreateMessage::new()
                .embed(embed)
                .components(vec![action_row]),
        )
        .await
    {
        format!("Failed to send close confirmation: {}", why)
    } else {
        "Closing ticket...".to_string()
    }
}
