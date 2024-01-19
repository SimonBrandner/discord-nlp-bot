pub mod container;
pub mod message;

use crate::store::SqlStore;
use sqlx::Error;

pub struct Processor {
    store: SqlStore,
}

impl Processor {
    pub fn new(store: SqlStore) -> Self {
        Self { store }
    }

    // TODO: Handle edits
    pub async fn add_message(&self, message: message::Message) {
        self.store.add_message(message).await;
    }

    pub async fn add_container(&self, container: container::Container) {
        self.store.add_container(container).await;
    }

    pub async fn get_first_and_last_message_id_in_container(
        &self,
        container_id: &str,
    ) -> Result<(String, String), Error> {
        let first = self.get_first_message_id_in_container(container_id).await?;
        let last = self.get_last_message_id_in_container(container_id).await?;

        Ok((first, last))
    }

    async fn get_last_message_id_in_container(&self, container_id: &str) -> Result<String, Error> {
        self.store
            .get_last_message_id_in_container(container_id)
            .await
    }

    async fn get_first_message_id_in_container(&self, container_id: &str) -> Result<String, Error> {
        self.store
            .get_first_message_id_in_container(container_id)
            .await
    }
}
