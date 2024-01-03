use crate::makers::make_message;
use nlp_bot_api::processor::{container, Processor};
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
        let processor = self.processor.lock().await;
        for guild_id in guilds {
            processor
                .add_container(container::Container {
                    container_id: guild_id.to_string(),
                    container_parent_id: String::from("discord"),
                })
                .await;

            let channels = { context.cache.guild(guild_id).unwrap().channels.clone() };

            for (channel_id, channel) in channels {
                if channel.kind != ChannelType::Text {
                    continue;
                }

                processor
                    .add_container(container::Container {
                        container_id: channel_id.to_string(),
                        container_parent_id: guild_id.to_string(),
                    })
                    .await;

                let messages = match channel
                    .messages(context.http(), GetMessages::new().limit(100))
                    .await
                {
                    Ok(messages) => messages,
                    Err(_error) => continue,
                };

                for discord_message in messages {
                    processor.add_message(make_message(discord_message)).await;
                }
            }
        }
    }
}
