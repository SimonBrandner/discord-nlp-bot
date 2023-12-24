use serenity::all::ChannelType;
use serenity::async_trait;
use serenity::builder::GetMessages;
use serenity::model::id::GuildId;
use serenity::prelude::Context;
use serenity::prelude::*;

pub struct DiscordBot {}

#[async_trait]
impl EventHandler for DiscordBot {
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
