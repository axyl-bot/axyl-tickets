use crate::{commands::*, config::Config};
use serenity::{all::*, async_trait, model::gateway::Ready, prelude::SerenityError};
use std::sync::Arc;

struct Handler {
    config: Arc<Config>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                let content = match command.data.name.as_str() {
                    "init" => init(&ctx, &command, &self.config).await,
                    "close" => close(&ctx, &command, &self.config)
                        .await
                        .unwrap_or_else(|e| format!("Error: {}", e)),
                    "adduser" => add_user(&ctx, &command, &self.config)
                        .await
                        .unwrap_or_else(|e| format!("Error: {}", e)),
                    "removeuser" => remove_user(&ctx, &command, &self.config)
                        .await
                        .unwrap_or_else(|e| format!("Error: {}", e)),
                    "setcategory" => set_category(&self.config, &command),
                    "setlogchannel" => set_log_channel(&self.config, &command),
                    _ => "Not implemented".to_string(),
                };

                if let Err(why) = command
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().content(content),
                        ),
                    )
                    .await
                {
                    println!("Cannot respond to slash command: {}", why);
                }
            }
            Interaction::Component(component) => {
                if component.data.custom_id == "open_ticket" {
                    if let Some(guild_id) = component.guild_id {
                        match guild_id.to_partial_guild(&ctx.http).await {
                            Ok(guild) => {
                                match create_ticket(&ctx, &component.user, &guild, &self.config)
                                    .await
                                {
                                    Ok(guild_channel) => {
                                        if let Err(why) = component
                                            .create_response(
                                                &ctx.http,
                                                CreateInteractionResponse::Message(
                                                    CreateInteractionResponseMessage::new()
                                                        .content(format!(
                                                            "Ticket created: {}",
                                                            guild_channel.mention()
                                                        ))
                                                        .ephemeral(true),
                                                ),
                                            )
                                            .await
                                        {
                                            println!("Error creating ticket: {}", why);
                                        }
                                    }
                                    Err(why) => println!("Error creating ticket: {}", why),
                                }
                            }
                            Err(why) => println!("Error fetching guild: {}", why),
                        }
                    }
                } else if component.data.custom_id == "close_ticket" {
                    if let Err(why) = close(&ctx, &component, &self.config).await {
                        println!("Error closing ticket: {}", why);
                    }
                } else if component.data.custom_id == "cancel_close" {
                    if let Err(why) = cancel_close(&ctx, &component).await {
                        println!("Error cancelling close: {}", why);
                    }
                }
            }
            _ => {}
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        match Command::get_global_commands(&ctx.http).await {
            Ok(commands) => {
                for command in commands {
                    if let Err(why) = Command::delete_global_command(&ctx.http, command.id).await {
                        println!(
                            "Failed to delete global command {}: {:?}",
                            command.name, why
                        );
                    } else {
                        println!("Deleted global command: {}", command.name);
                    }
                }
            }
            Err(why) => println!("Failed to get global commands: {:?}", why),
        }

        let commands = vec![
            CreateCommand::new("init").description("Initialize the ticket embed"),
            CreateCommand::new("close").description("Close the current ticket"),
            CreateCommand::new("adduser")
                .description("Add a user to the ticket")
                .add_option(
                    CreateCommandOption::new(CommandOptionType::User, "user", "The user to add")
                        .required(true),
                ),
            CreateCommand::new("removeuser")
                .description("Remove a user from the ticket")
                .add_option(
                    CreateCommandOption::new(CommandOptionType::User, "user", "The user to remove")
                        .required(true),
                ),
            CreateCommand::new("setcategory")
                .description("Set the category ID for tickets")
                .add_option(
                    CreateCommandOption::new(CommandOptionType::String, "id", "The category ID")
                        .required(true),
                ),
            CreateCommand::new("setlogchannel")
                .description("Set the log channel ID")
                .add_option(
                    CreateCommandOption::new(CommandOptionType::String, "id", "The log channel ID")
                        .required(true),
                ),
        ];

        match Command::set_global_commands(&ctx.http, commands).await {
            Ok(_) => println!("Global slash commands registered successfully"),
            Err(why) => println!("Failed to register global slash commands: {:?}", why),
        }

        ctx.set_presence(
            Some(
                ActivityData::streaming("twitch.tv/axylprojects", "https://twitch.tv/axylprojects")
                    .expect("Failed to create streaming activity"),
            ),
            OnlineStatus::DoNotDisturb,
        )
    }
}

pub async fn run_bot() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(Config::new());
    let token = &config.token;
    let intents =
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILDS;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler {
            config: config.clone(),
        })
        .await?;

    client.start().await?;

    Ok(())
}

async fn cancel_close(
    ctx: &Context,
    component: &ComponentInteraction,
) -> Result<(), SerenityError> {
    let mut message = component.message.clone();
    message
        .edit(&ctx.http, EditMessage::new().components(vec![]))
        .await?;

    component
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("Ticket closure cancelled.")
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}

fn set_category(config: &Arc<Config>, command: &CommandInteraction) -> String {
    if let Some(id) = command
        .data
        .options
        .get(0)
        .and_then(|opt| opt.value.as_str())
    {
        if let Ok(category_id) = id.parse::<u64>() {
            config.set_category_id(category_id);
            format!("Category ID set to {}", category_id)
        } else {
            "Invalid category ID provided".to_string()
        }
    } else {
        "No category ID provided".to_string()
    }
}

fn set_log_channel(config: &Arc<Config>, command: &CommandInteraction) -> String {
    if let Some(id) = command
        .data
        .options
        .get(0)
        .and_then(|opt| opt.value.as_str())
    {
        if let Ok(log_channel_id) = id.parse::<u64>() {
            config.set_log_channel_id(log_channel_id);
            format!("Log channel ID set to {}", log_channel_id)
        } else {
            "Invalid log channel ID provided".to_string()
        }
    } else {
        "No log channel ID provided".to_string()
    }
}
