use crate::{commands::*, config::Config};
use serenity::{all::*, async_trait, model::gateway::Ready, prelude::SerenityError};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                let content = match command.data.name.as_str() {
                    "init" => init(&ctx, &command).await,
                    "close" => close(&ctx, &command)
                        .await
                        .unwrap_or_else(|e| format!("Error: {}", e)),
                    "adduser" => add_user(&ctx, &command).await,
                    "removeuser" => remove_user(&ctx, &command).await,
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
                            Ok(guild) => match create_ticket(&ctx, &component.user, &guild).await {
                                Ok(channel) => {
                                    if let Err(why) = component
                                        .create_response(
                                            &ctx.http,
                                            CreateInteractionResponse::Message(
                                                CreateInteractionResponseMessage::new()
                                                    .content(format!(
                                                        "Ticket created: {}",
                                                        channel.mention()
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
                            },
                            Err(why) => println!("Error fetching guild: {}", why),
                        }
                    }
                } else if component.data.custom_id == "close_ticket" {
                    if let Err(why) = close(&ctx, &component).await {
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

        let guild_id = GuildId::new(Config::get().guild_id);

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
        ];

        match guild_id.set_commands(&ctx.http, commands).await {
            Ok(_) => println!("Slash commands registered successfully"),
            Err(why) => println!("Failed to register slash commands: {:?}", why),
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
    let config = Config::get();
    let token = &config.token;
    let intents =
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILDS;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
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
