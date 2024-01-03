use crate::makers::make_message;
use nlp_bot_api::processor::Processor;
use serenity::all::{ChannelType, GatewayIntents};
use serenity::builder::GetMessages;
use serenity::client::EventHandler;
use serenity::http::CacheHttp;
use serenity::model::id::GuildId;
use serenity::prelude::Context;
use serenity::{async_trait, Client};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn start_bot(bot: Bot, token: String) {
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(token, intents)
        .event_handler(bot)
        .await
        .expect("Failed to build client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

pub struct Bot {
    processor: Arc<Mutex<Processor>>,
}

impl Bot {
    pub fn new(processor: Arc<Mutex<Processor>>) -> Self {
        Self { processor }
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn cache_ready(&self, context: Context, guilds: Vec<GuildId>) {
        log::info!("Discord cache is ready...");
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
                    Err(_error) => continue,
                };

                let processor = self.processor.lock().await;
                for discord_message in messages {
                    println!("Message: {}", discord_message.content);
                    processor.add_message(make_message(discord_message)).await;
                }
            }
        }
    }
}
