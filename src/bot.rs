use crate::{commands::*, config::Config};
use serenity::{all::*, async_trait, model::gateway::Ready};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
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
