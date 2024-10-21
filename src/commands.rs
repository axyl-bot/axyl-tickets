use crate::config::Config;
use serenity::{
    all::*,
    builder::{CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, CreateMessage},
};
use tokio::time::{sleep, Duration};

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

    let message = command
        .channel_id
        .send_message(
            &ctx.http,
            CreateMessage::new()
                .embed(embed)
                .components(vec![action_row]),
        )
        .await;

    if let Err(why) = message {
        return format!("Failed to send close confirmation: {}", why);
    }

    let message = message.unwrap();

    sleep(Duration::from_secs(5)).await;

    if let Ok(updated_message) = message.channel_id.message(&ctx.http, message.id).await {
        if !updated_message.components.is_empty() {
            if let Err(why) = command.channel_id.delete(&ctx.http).await {
                format!("Failed to close the ticket: {}", why)
            } else {
                "Ticket closed successfully.".to_string()
            }
        } else {
            "Ticket closure was cancelled.".to_string()
        }
    } else {
        "Failed to check ticket status.".to_string()
    }
}
