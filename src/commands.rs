use crate::config::Config;
use crate::logging::log_ticket_action;
use serenity::{
    all::*,
    builder::{CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, CreateMessage},
    prelude::SerenityError,
};
use std::borrow::Cow;
use std::error::Error;
use std::fmt;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct TicketError(Cow<'static, str>);

impl fmt::Display for TicketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for TicketError {}

impl From<TicketError> for SerenityError {
    fn from(error: TicketError) -> Self {
        SerenityError::Other(Box::leak(error.0.into_owned().into_boxed_str()))
    }
}

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

pub async fn create_ticket(
    ctx: &Context,
    user: &User,
    guild: &PartialGuild,
) -> Result<GuildChannel, SerenityError> {
    let config = Config::get();
    let channel_name = format!("ticket-{}", user.name.to_lowercase());

    let everyone_role = guild
        .roles
        .values()
        .find(|r| r.name == "@everyone")
        .unwrap()
        .id;

    let channel_builder = CreateChannel::new(channel_name.clone())
        .kind(ChannelType::Text)
        .category(ChannelId::new(config.category_id))
        .permissions(vec![
            PermissionOverwrite {
                allow: Permissions::VIEW_CHANNEL
                    | Permissions::SEND_MESSAGES
                    | Permissions::READ_MESSAGE_HISTORY,
                deny: Permissions::empty(),
                kind: PermissionOverwriteType::Member(user.id),
            },
            PermissionOverwrite {
                allow: Permissions::empty(),
                deny: Permissions::VIEW_CHANNEL,
                kind: PermissionOverwriteType::Role(everyone_role),
            },
        ]);

    let guild_channel = guild.create_channel(&ctx.http, channel_builder).await?;

    let embed = CreateEmbed::new()
        .title("New Ticket")
        .description(
            "Please describe the reasoning for opening this ticket, include any information you \
            think may be relevant such as proof, other third parties and so on.\n\n\
            Use `/adduser` if you want to add another user.\n\
            Do not add them if they are the subject of a report, as they can close the ticket.\n\n\
            Please close the ticket using `/close` when you feel that the issue is resolved.",
        )
        .color(0x2b2d31)
        .footer(CreateEmbedFooter::new(format!(
            "Channel: #{}",
            channel_name
        )));

    let close_button = CreateButton::new("close_ticket")
        .label("Close Ticket")
        .style(ButtonStyle::Danger);

    let action_row = CreateActionRow::Buttons(vec![close_button]);

    guild_channel
        .send_message(
            &ctx.http,
            CreateMessage::new()
                .content(format!(
                    "Thank you for opening a moderation ticket {}",
                    user.mention()
                ))
                .embed(embed)
                .components(vec![action_row]),
        )
        .await?;

    log_ticket_action(ctx, "Opened", user, &guild_channel).await?;
    Ok(guild_channel)
}

pub async fn close(
    ctx: &Context,
    interaction: &impl InteractionContext,
) -> Result<String, SerenityError> {
    let channel_id = interaction.channel_id();

    let embed = CreateEmbed::new()
        .title("Closing Ticket")
        .description("This ticket will be closed in 5 seconds. Click the button below to cancel.")
        .color(0xff0000);

    let button = CreateButton::new("cancel_close")
        .label("Cancel")
        .style(ButtonStyle::Danger);

    let action_row = CreateActionRow::Buttons(vec![button]);

    let message = channel_id
        .send_message(
            &ctx.http,
            CreateMessage::new()
                .embed(embed)
                .components(vec![action_row]),
        )
        .await?;

    sleep(Duration::from_secs(5)).await;

    if let Ok(updated_message) = message.channel_id.message(&ctx.http, message.id).await {
        if !updated_message.components.is_empty() {
            if let Ok(Channel::Guild(guild_channel)) = channel_id.to_channel(&ctx).await {
                log_ticket_action(ctx, "Closed", interaction.user(), &guild_channel).await?;
                channel_id.delete(&ctx.http).await?;
                Ok("Ticket closed successfully.".to_string())
            } else {
                Ok("Failed to close ticket: not a guild channel.".to_string())
            }
        } else {
            Ok("Ticket closure was cancelled.".to_string())
        }
    } else {
        Ok("Failed to check ticket status.".to_string())
    }
}

pub trait InteractionContext {
    fn channel_id(&self) -> ChannelId;
    fn user(&self) -> &User;
}

impl InteractionContext for CommandInteraction {
    fn channel_id(&self) -> ChannelId {
        self.channel_id
    }

    fn user(&self) -> &User {
        &self.user
    }
}

impl InteractionContext for ComponentInteraction {
    fn channel_id(&self) -> ChannelId {
        self.channel_id
    }

    fn user(&self) -> &User {
        &self.user
    }
}

pub async fn add_user(
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<String, SerenityError> {
    if let Some(_guild_id) = command.guild_id {
        if let Some(user) = command.data.resolved.users.values().next() {
            if let Ok(channel) = command.channel_id.to_channel(&ctx).await {
                if let Channel::Guild(guild_channel) = channel {
                    if let Ok(()) = guild_channel
                        .create_permission(
                            &ctx.http,
                            PermissionOverwrite {
                                allow: Permissions::VIEW_CHANNEL
                                    | Permissions::SEND_MESSAGES
                                    | Permissions::READ_MESSAGE_HISTORY,
                                deny: Permissions::empty(),
                                kind: PermissionOverwriteType::Member(user.id),
                            },
                        )
                        .await
                    {
                        log_ticket_action(ctx, "User Added", user, &guild_channel).await?;
                        Ok(format!("User {} has been added to the ticket.", user.name))
                    } else {
                        Err(TicketError(Cow::Borrowed("Failed to add user to the ticket.")).into())
                    }
                } else {
                    Err(TicketError(Cow::Borrowed(
                        "This command can only be used in a server channel.",
                    ))
                    .into())
                }
            } else {
                Err(TicketError(Cow::Borrowed("Failed to fetch channel information.")).into())
            }
        } else {
            Err(TicketError(Cow::Borrowed("Please mention a user to add to the ticket.")).into())
        }
    } else {
        Err(TicketError(Cow::Borrowed("This command can only be used in a server.")).into())
    }
}

pub async fn remove_user(
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<String, SerenityError> {
    if let Some(_guild_id) = command.guild_id {
        if let Some(user) = command.data.resolved.users.values().next() {
            if let Ok(channel) = command.channel_id.to_channel(&ctx).await {
                if let Channel::Guild(guild_channel) = channel {
                    if let Ok(()) = guild_channel
                        .delete_permission(&ctx.http, PermissionOverwriteType::Member(user.id))
                        .await
                    {
                        log_ticket_action(ctx, "User Removed", user, &guild_channel).await?;
                        Ok(format!(
                            "User {} has been removed from the ticket.",
                            user.name
                        ))
                    } else {
                        Err(
                            TicketError(Cow::Borrowed("Failed to remove user from the ticket."))
                                .into(),
                        )
                    }
                } else {
                    Err(TicketError(Cow::Borrowed(
                        "This command can only be used in a server channel.",
                    ))
                    .into())
                }
            } else {
                Err(TicketError(Cow::Borrowed("Failed to fetch channel information.")).into())
            }
        } else {
            Err(TicketError(Cow::Borrowed(
                "Please mention a user to remove from the ticket.",
            ))
            .into())
        }
    } else {
        Err(TicketError(Cow::Borrowed("This command can only be used in a server.")).into())
    }
}
