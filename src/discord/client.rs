use super::bot::DiscordBot;
use serenity::client::ClientBuilder;
use serenity::prelude::*;

pub struct DiscordClient {
    client: ClientBuilder,
}

impl DiscordClient {
    pub fn new(token: String) -> Self {
        let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
        let bot = DiscordBot {};

        Self {
            client: Client::builder(token, intents).event_handler(bot),
        }
    }

    pub async fn connect(self) {
        let mut client = self.client.await.expect("Failed to build client");

        // start listening for events by starting a single shard
        if let Err(why) = client.start().await {
            println!("Client error: {why:?}");
        }
    }
}
