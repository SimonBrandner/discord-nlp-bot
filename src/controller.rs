use crate::discord::client::DiscordClient;

use super::store::sql::SqlStore;

pub struct Controller {
    sql_store: SqlStore,
    discord_client: DiscordClient,
}

impl Controller {
    pub fn new(sql_store: SqlStore, discord_client: DiscordClient) -> Self {
        Self {
            sql_store,
            discord_client,
        }
    }

    pub async fn update(self) {
        self.discord_client.connect().await;
    }
}
