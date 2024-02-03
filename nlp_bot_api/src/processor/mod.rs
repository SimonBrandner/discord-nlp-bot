pub mod container;
pub mod entry;
pub mod ngram;

use crate::store;

const ENTRY_LIMIT: u32 = 100;

#[derive(Debug)]
pub enum Error {
    DatabaseError(sqlx::Error),
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError(err)
    }
}

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
            let entries = self
                .store
                .get_entries_without_cached_ngrams(ENTRY_LIMIT, start_index)
                .await?;

            start_index = match entries.last() {
                Some(entry) => Some(entry.entry_id.clone()),
                None => break,
            };

            let entry_ids: Vec<String> = entries.iter().map(|m| m.entry_id.clone()).collect();
            let ngrams = entry::Entry::get_ngrams_from_entries_slice(entries.as_slice());

            self.store.add_ngrams(ngrams.as_slice()).await?;
            self.store.mark_entry_as_ngrams_cached(&entry_ids).await?;
        }

        log::info!("Cached ngrams for all entries.");
        Ok(())
    }

    pub async fn add_entry(&self, entry: entry::Entry) -> Result<(), Error> {
        self.add_entries([entry].as_slice()).await
    }

    pub async fn add_entries(&self, entries: &[entry::Entry]) -> Result<(), Error> {
        let ngrams = entry::Entry::get_ngrams_from_entries_slice(entries);

        self.store.add_ngrams(ngrams.as_slice()).await?;
        self.store.add_entries(entries, true).await?;

        Ok(())
    }

    pub async fn add_container(&self, container: &container::Container) -> Result<(), Error> {
        self.store.add_container(container).await?;

        Ok(())
    }

    pub async fn get_first_and_last_entry_id_in_container(
        &self,
        container_id: &str,
    ) -> Result<(String, String), Error> {
        let first = self.get_first_entry_id_in_container(container_id).await?;
        let last = self.get_last_entry_id_in_container(container_id).await?;

        Ok((first, last))
    }

    async fn get_last_entry_id_in_container(&self, container_id: &str) -> Result<String, Error> {
        let last_entry_id = self
            .store
            .get_last_entry_id_in_container(container_id)
            .await?;

        Ok(last_entry_id)
    }

    async fn get_first_entry_id_in_container(&self, container_id: &str) -> Result<String, Error> {
        let first_entry_id = self
            .store
            .get_first_entry_id_in_container(container_id)
            .await?;

        Ok(first_entry_id)
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
