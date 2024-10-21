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
                    "close" => close(&ctx, &command).await,
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
                    if let Err(why) = create_ticket(&ctx, &component).await {
                        println!("Error creating ticket: {}", why);
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
        );
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

async fn create_ticket(
    ctx: &Context,
    component: &ComponentInteraction,
) -> Result<(), SerenityError> {
    let guild = component.guild_id.unwrap();
    let config = Config::get();
    let user = &component.user;
    let channel_name = format!("ticket-{}", user.name.to_lowercase());

    let everyone_role = guild
        .roles(&ctx.http)
        .await?
        .values()
        .find(|r| r.name == "@everyone")
        .unwrap()
        .id;

    let channel_builder = CreateChannel::new(channel_name)
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

    let ticket_channel = guild.create_channel(&ctx.http, channel_builder).await?;

    component
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(format!("Ticket created: {}", ticket_channel.mention())),
            ),
        )
        .await?;

    Ok(())
}

async fn cancel_close(
    ctx: &Context,
    component: &ComponentInteraction,
) -> Result<(), SerenityError> {
    component
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new().content("Ticket closure cancelled."),
            ),
        )
        .await?;

    Ok(())
}
