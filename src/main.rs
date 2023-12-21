mod config;

use clap::Parser;

use serenity::all::ChannelType;
use serenity::async_trait;
use serenity::builder::GetMessages;
use serenity::model::id::GuildId;
use serenity::prelude::Context;
use serenity::prelude::*;

#[derive(clap::Parser, Debug)]
struct CommandLineArguments {
    /// The path to the configuration file
    #[arg(short, long, default_value = "./config.json")]
    configuration_file: String,
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, context: Context, guilds: Vec<GuildId>) {
        for guild_id in guilds {
            let channels = { context.cache.guild(guild_id).unwrap().channels.clone() };

            for (_channel_id, channel) in channels {
                if channel.kind != ChannelType::Text {
                    continue;
                }

                let guild_name = match channel.guild(&context.cache) {
                    Some(guild) => guild.name.clone(),
                    None => String::from("No guild"),
                };

                println!("Guild: {} | Channel: {}", guild_name, channel.name);

                let messages = match channel
                    .messages(context.http(), GetMessages::new().limit(100))
                    .await
                {
                    Ok(messages) => messages,
                    Err(_error) => Vec::new(),
                };

                for message in messages {
                    println!("Message: {}", message.content)
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let command_line_arguments = CommandLineArguments::parse();
    let configuration =
        config::read_configuration_from_file(command_line_arguments.configuration_file);

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(configuration.discord_token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
