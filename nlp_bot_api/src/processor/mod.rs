pub mod container;
pub mod message;
pub mod ngram;

use crate::store;
use sqlx::Error;

const MESSAGE_LIMIT: u32 = 100;

pub struct Processor {
    store: store::Sql,
}

impl Processor {
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(store: store::Sql) -> Self {
        Self { store }
    }

    pub async fn cache_ngrams(&self) -> Result<(), Error> {
        let mut start_index: Option<String> = None;
        loop {
            let messages = match self
                .store
                .get_messages_without_cached_ngrams(MESSAGE_LIMIT, start_index)
                .await
            {
                Ok(messages) => messages,
                Err(e) => {
                    return Err(e);
                }
            };

            start_index = match messages.last() {
                Some(message) => Some(message.message_id.clone()),
                None => break,
            };

            let message_ids: Vec<String> = messages.iter().map(|m| m.message_id.clone()).collect();
            let ngrams = message::Message::get_ngrams_from_message_slice(messages.as_slice());

            self.store.add_ngrams(ngrams.as_slice()).await;
            self.store.mark_message_ngrams_cached(&message_ids).await;
        }

        log::info!("Cached ngrams for all messages");
        Ok(())
    }

    pub async fn add_message(&self, message: message::Message) {
        self.add_messages([message].as_slice()).await;
    }

    pub async fn add_messages(&self, messages: &[message::Message]) {
        let ngrams = message::Message::get_ngrams_from_message_slice(messages);

        self.store.add_ngrams(ngrams.as_slice()).await;
        self.store.add_messages(messages, true).await;
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

    /// This returns the parent `container_id` as well.
    async fn get_child_container_ids(&self, container_id: &str) -> Result<Vec<String>, Error> {
        let mut container_ids = vec![container_id.to_string()];
        let mut container_ids_to_explore = vec![container_id.to_string()];

        loop {
            if container_ids_to_explore.is_empty() {
                break;
            }

            let mut new_container_ids_to_explore = Vec::new();
            for container_id_to_explore in container_ids_to_explore {
                let child_container_ids = self
                    .store
                    .get_child_container_ids(&container_id_to_explore)
                    .await?;

                container_ids.extend(child_container_ids.clone());
                new_container_ids_to_explore.extend(child_container_ids);
            }

            container_ids_to_explore = new_container_ids_to_explore;
        }

        Ok(container_ids)
    }

    pub async fn get_ngram_count_in_container(&self, container_id: &str) -> Result<i32, Error> {
        let child_container_ids = self.get_child_container_ids(container_id).await?;

        let mut ngram_count = 0;
        for child_container_id in child_container_ids {
            ngram_count += self
                .store
                .get_ngram_count_in_container(&child_container_id)
                .await?;
        }

        Ok(ngram_count)
    }

    pub async fn get_entries_count_in_container(&self, container_id: &str) -> Result<i32, Error> {
        let child_container_ids = self.get_child_container_ids(container_id).await?;

        let mut entries_count = 0;
        for child_container_id in child_container_ids {
            entries_count += self
                .store
                .get_entries_count_in_container(&child_container_id)
                .await?;
        }

        Ok(entries_count)
    }
}
