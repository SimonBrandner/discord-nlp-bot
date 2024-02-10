use crate::commands::{on_error, on_ngrams_by_count, SharedCommandData};
use crate::makers::make_entry;
use nlp_bot_api::processor::entry::Entry;
use nlp_bot_api::processor::{container, Processor};
use poise::{Framework, FrameworkOptions, PrefixFrameworkOptions};
use serenity::all::{ChannelType, GatewayIntents, Guild, GuildChannel, Message, MessageId};
use serenity::builder::GetMessages;
use serenity::client::EventHandler;
use serenity::http::CacheHttp;
use serenity::model::id::GuildId;
use serenity::prelude::Context;
use serenity::{async_trait, Client, Error};
use std::sync::Arc;
use std::time::Duration;

const MESSAGE_LIMIT: u8 = 100;

#[derive(PartialEq, Debug)]
enum PaginationDirection {
    /// From old messages to new ones
    Up { message_id: Option<MessageId> },
    /// From new messages to new ones
    Down { message_id: MessageId },
}

pub async fn start(bot: Bot, processor: Arc<Processor>, token: String) {
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let options = FrameworkOptions {
        commands: vec![on_ngrams_by_count()],
        prefix_options: PrefixFrameworkOptions {
            prefix: Some("/nlp".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        ..Default::default()
    };
    let framework = Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(SharedCommandData {
                    processor: processor.clone(),
                })
            })
        })
        .options(options)
        .build();

    let mut client = Client::builder(token, intents)
        .event_handler(bot)
        .framework(framework)
        .await
        .expect("Failed to build client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}

pub struct Bot {
    processor: Arc<Processor>,
}

impl Bot {
    pub fn new(processor: Arc<Processor>) -> Self {
        Self { processor }
    }

    async fn paginate(&self, context: &Context, channel: &GuildChannel) {
        let first_and_last_messages_id = self
            .processor
            .get_first_and_last_entry_id_in_container(&channel.id.to_string())
            .await;

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
            "Paginating in container {} ({}) in direction {:?}",
            channel.name,
            channel.id,
            direction
        );

        let mut get_messages: GetMessages = GetMessages::new().limit(MESSAGE_LIMIT);
        get_messages = match direction {
            PaginationDirection::Down { message_id } => get_messages.after(message_id),
            PaginationDirection::Up { message_id } => {
                message_id.map_or(get_messages, |message_id| get_messages.before(message_id))
            }
        };

        loop {
            let messages = channel.messages(context.http(), get_messages).await?;

            let last_message_id = match messages.last() {
                Some(message) => message.id,
                None => break,
            };

            let entries: Vec<Entry> = messages.iter().map(make_entry).collect();
            self.processor
                .add_entries(entries.as_slice())
                .await
                .expect("Failed to add entries!");

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
        self.processor
            .add_container(&container::Container {
                container_id: channel.id.to_string(),
                container_parent_id: channel.guild_id.to_string(),
            })
            .await
            .expect("Failed to add container for channel!");

        self.paginate(context, channel).await;
    }

    async fn process_guild(&self, context: &Context, guild: &Guild) {
        self.processor
            .add_container(&container::Container {
                container_id: guild.id.to_string(),
                container_parent_id: String::from("discord"),
            })
            .await
            .expect("Failed to add container for guild!");

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
        self.processor
            .add_entry(make_entry(&new_message))
            .await
            .expect("Failed to add entry!");
    }

    async fn cache_ready(&self, context: Context, guilds: Vec<GuildId>) {
        log::info!("Discord cache is ready...");
        for guild_id in guilds {
            let guild;
            if let Some(g) = context.cache.guild(guild_id) {
                guild = g.clone();
            } else {
                log::warn!("Failed to get guild: {}", guild_id);
                continue;
            }
            self.process_guild(&context, &guild).await;
        }

        log::info!("Read all containers!");
    }
}
