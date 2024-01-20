use crate::makers::make_message;
use nlp_bot_api::processor::{container, Processor};
use serenity::all::{ChannelType, GatewayIntents, Guild, GuildChannel, Message, MessageId};
use serenity::builder::GetMessages;
use serenity::client::EventHandler;
use serenity::http::CacheHttp;
use serenity::model::id::GuildId;
use serenity::prelude::Context;
use serenity::{async_trait, Client, Error};
use std::sync::Arc;
use tokio::sync::Mutex;

const MESSAGE_LIMIT: u8 = 100;

#[derive(PartialEq, Debug)]
enum PaginationDirection {
    /// From old messages to new ones
    Up { message_id: Option<MessageId> },
    /// From new messages to new ones
    Down { message_id: MessageId },
}

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

    async fn paginate(&self, context: &Context, channel: &GuildChannel) {
        let processor = self.processor.lock().await;
        let first_and_last_messages_id = processor
            .get_first_and_last_message_id_in_container(&channel.id.to_string())
            .await;
        drop(processor);

        match first_and_last_messages_id {
            Ok((first_message_id, last_message_id)) => {
                self.paginate_in_direction(
                    context,
                    channel,
                    PaginationDirection::Up {
                        message_id: Some(MessageId::new(
                            first_message_id
                                .parse()
                                .expect("The database contains a message ID that is not a number!"),
                        )),
                    },
                )
                .await
                .expect("Failed to paginate up");

                self.paginate_in_direction(
                    context,
                    channel,
                    PaginationDirection::Down {
                        message_id: MessageId::new(
                            last_message_id
                                .parse()
                                .expect("The database contains a message ID that is not a number!"),
                        ),
                    },
                )
                .await
                .expect("Failed to paginate down");
            }
            Err(_e) => {
                self.paginate_in_direction(
                    context,
                    channel,
                    PaginationDirection::Up { message_id: None },
                )
                .await
                .expect("Failed to paginate up from bottom");
            }
        };
    }

    async fn paginate_in_direction(
        &self,
        context: &Context,
        channel: &GuildChannel,
        direction: PaginationDirection,
    ) -> Result<(), Error> {
        log::info!(
            "Paginating in container {} in direction {:?}",
            channel.id,
            direction
        );

        let mut get_messages: GetMessages = GetMessages::new().limit(MESSAGE_LIMIT);
        get_messages = match direction {
            PaginationDirection::Down { message_id } => get_messages.after(message_id),
            PaginationDirection::Up { message_id } => match message_id {
                Some(message_id) => get_messages.before(message_id),
                None => get_messages,
            },
        };

        loop {
            let messages = match channel.messages(context.http(), get_messages).await {
                Ok(messages) => messages,
                Err(error) => return Err(error),
            };

            let last_message_id = match messages.last() {
                Some(message) => message.id,
                None => break,
            };

            let processor = self.processor.lock().await;
            for discord_message in messages {
                processor.add_message(make_message(discord_message)).await;
            }

            get_messages = match direction {
                PaginationDirection::Up { message_id: _id } => get_messages.before(last_message_id),
                PaginationDirection::Down { message_id: _id } => {
                    get_messages.after(last_message_id)
                }
            };
        }

        Ok(())
    }

    async fn process_channel(&self, context: &Context, channel: &GuildChannel) {
        let processor = self.processor.lock().await;
        processor
            .add_container(container::Container {
                container_id: channel.id.to_string(),
                container_parent_id: channel.guild_id.to_string(),
            })
            .await;
        drop(processor);

        self.paginate(context, channel).await;
    }

    async fn process_guild(&self, context: &Context, guild: &Guild) {
        let processor = self.processor.lock().await;
        processor
            .add_container(container::Container {
                container_id: guild.id.to_string(),
                container_parent_id: String::from("discord"),
            })
            .await;
        drop(processor);

        for channel in guild.channels.values() {
            if channel.kind == ChannelType::Text {
                self.process_channel(context, channel).await;
            }
        }
    }
}

#[async_trait]
impl EventHandler for Bot {
    // TODO: Handle updates
    // TODO: Model relations (replies)

    async fn message(&self, _context: Context, new_message: Message) {
        let processor = self.processor.lock().await;
        processor.add_message(make_message(new_message)).await;
    }

    async fn cache_ready(&self, context: Context, guilds: Vec<GuildId>) {
        log::info!("Discord cache is ready...");
        for guild_id in guilds {
            let guild = match context.cache.guild(guild_id) {
                Some(guild) => guild.clone(),
                None => {
                    log::warn!("Failed to get guild: {}", guild_id);
                    continue;
                }
            };
            self.process_guild(&context, &guild).await;
        }

        log::info!("Read all containers!")
    }
}
