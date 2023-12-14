mod config;

use std::collections::hash_map::Values;

use clap::Parser;

use serenity::all::{ChannelId, ChannelType, GuildChannel};
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

async fn get_channels(context: &Context, channels: Values<'_, ChannelId, GuildChannel>) {
    println!("Length {}", channels.len());

    for channel in channels {
        if channel.kind != ChannelType::Text {
            continue;
        }

        let guild_name: String;
        match channel.guild(&context.cache) {
            Some(guild) => guild_name = guild.name.clone(),
            None => guild_name = String::from("No guild"),
        }

        println!("Channel: {}::{}", channel.name, guild_name);

        match channel
            .messages(context.http(), GetMessages::new().limit(100))
            .await
        {
            Ok(messages) => {
                for message in messages {
                    println!("Message: {}", message.content)
                }
            }
            Err(error) => println!("{}", error.to_string()),
        }
    }
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, context: Context, guilds: Vec<GuildId>) {
        for guild_id in guilds {
            let channels = context.cache.guild_channels(guild_id).unwrap();

            /*match context.cache.guild_channels(guild_id) {
                Some(channels) => _channels = channels.values(),
                None => println!("No channels"),
            }*/

            get_channels(&context, channels.values()).await;
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
