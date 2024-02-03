use crate::processor::container;
use crate::processor::entry;
use crate::processor::entry::Entry;
use crate::processor::ngram::Ngram;
use sqlx::migrate;
use sqlx::migrate::MigrateError;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::Error;
use sqlx::QueryBuilder;
use sqlx::{migrate::MigrateDatabase, Connection, Sqlite, SqliteConnection};
use tokio::sync::Mutex;

/// Limit as per <https://stackoverflow.com/a/15860818/10822785>
const CHUNK_SIZE: usize = 500;

pub struct Sql {
    connection: Mutex<SqliteConnection>,
}

impl Sql {
    pub async fn new(file_path: &str) -> Result<Self, Error> {
        if !Sqlite::database_exists(file_path).await.unwrap_or(false) {
            log::info!("Database does not exist - creating...");
            Sqlite::create_database(file_path).await?;
            log::info!("Database created successfully!");
        }

        let database_connection =
            SqliteConnection::connect(format!("sqlite://{}", file_path).as_str()).await?;

        let sql = Self {
            connection: Mutex::new(database_connection),
        };

        sql.migrate().await?;

        Ok(sql)
    }

    async fn migrate(&self) -> Result<(), MigrateError> {
        migrate!("./src/migrations")
            .run(&mut *self.connection.lock().await)
            .await
    }

    pub async fn mark_entry_as_ngrams_cached(
        &self,
        entry_ids: &[String],
    ) -> Result<SqliteQueryResult, Error> {
        let entry_ids_string = entry_ids.join(",");

        sqlx::query!(
            "UPDATE entries SET ngrams_cached=true WHERE entry_id IN (?);",
            entry_ids_string
        )
        .execute(&mut *self.connection.lock().await)
        .await
    }

    pub async fn add_ngrams(&self, ngrams: &[Ngram]) -> Result<(), Error> {
        if ngrams.is_empty() {
            return Ok(());
        }

        for ngrams_chunk in ngrams.chunks(CHUNK_SIZE) {
            let mut query_builder = QueryBuilder::new(
                "INSERT INTO ngrams (count, content, time, sender_id, container_id) ",
            );
            query_builder.push_values(ngrams_chunk, |mut query_builder, ngram| {
                query_builder
                    .push_bind(1)
                    .push_bind(ngram.content.clone())
                    .push_bind(ngram.time)
                    .push_bind(ngram.sender_id.clone())
                    .push_bind(ngram.container_id.clone());
            });
            query_builder.push(
                " ON CONFLICT (content, time, sender_id, container_id) DO UPDATE SET count = count + 1;",
            );
            query_builder
                .build()
                .execute(&mut *self.connection.lock().await)
                .await?;
        }

        Ok(())
    }

    pub async fn add_entries(
        &self,
        entries: &[entry::Entry],
        ngrams_cached: bool,
    ) -> Result<(), Error> {
        if entries.is_empty() {
            return Ok(());
        }

        for entries_chunk in entries.chunks(CHUNK_SIZE) {
            let mut query_builder = QueryBuilder::new(
                "INSERT INTO entries (entry_id, content, sender_id, container_id, unix_timestamp, ngrams_cached) ",
            );
            query_builder.push_values(entries_chunk, |mut query_builder, entry| {
                query_builder
                    .push_bind(entry.entry_id.clone())
                    .push_bind(entry.content.clone())
                    .push_bind(entry.sender_id.clone())
                    .push_bind(entry.container_id.clone())
                    .push_bind(entry.unix_timestamp)
                    .push_bind(ngrams_cached);
            });
            query_builder
                .build()
                .execute(&mut *self.connection.lock().await)
                .await?;
        }

        Ok(())
    }

    pub async fn add_container(
        &self,
        container: &container::Container,
    ) -> Result<SqliteQueryResult, Error> {
        sqlx::query!(
            "INSERT INTO containers (container_id, container_parent_id) VALUES (?, ?);",
            container.container_id,
            container.container_parent_id
        )
        .execute(&mut *self.connection.lock().await)
        .await
    }

    pub async fn get_last_entry_id_in_container(
        &self,
        container_id: &str,
    ) -> Result<String, Error> {
        let result =  sqlx::query!(
            "SELECT entry_id FROM entries WHERE container_id=? ORDER BY unix_timestamp DESC LIMIT 1;",
            container_id
        )
        .fetch_one(&mut *self.connection.lock().await).await?;

        Ok(result.entry_id)
    }

    pub async fn get_first_entry_id_in_container(
        &self,
        container_id: &str,
    ) -> Result<String, Error> {
        let result =  sqlx::query!(
            "SELECT entry_id FROM entries WHERE container_id=? ORDER BY unix_timestamp ASC LIMIT 1;",
            container_id
        )
        .fetch_one(&mut *self.connection.lock().await).await?;

        Ok(result.entry_id)
    }

    pub async fn get_entries_without_cached_ngrams(
        &self,
        limit: u32,
        start_id: Option<String>,
    ) -> Result<Vec<Entry>, Error> {
        match start_id {
            Some(start_id) => {
                sqlx::query!(
                    "
                    SELECT * FROM entries WHERE ngrams_cached=false AND unix_timestamp < (
                        SELECT unix_timestamp FROM entries WHERE entry_id=?
                    ) ORDER BY unix_timestamp DESC LIMIT ?;
                    ",
                    start_id,
                    limit
                )
                .fetch_all(&mut *self.connection.lock().await)
                .await
                .map(|rows| {
                    rows.into_iter()
                        .map(|row| Entry {
                            entry_id: row.entry_id,
                            container_id: row.container_id,
                            sender_id: row.sender_id,
                            unix_timestamp: row.unix_timestamp,
                            content: row.content,
                        })
                        .collect()
                })
            },
            None => {
                sqlx::query!(
                    "SELECT * FROM entries WHERE ngrams_cached=false ORDER BY unix_timestamp DESC LIMIT ?;",
                    limit
                )
                .fetch_all(&mut *self.connection.lock().await)
                .await
                .map(|rows| {
                    rows.into_iter()
                        .map(|row| Entry {
                            entry_id: row.entry_id,
                            container_id: row.container_id,
                            sender_id: row.sender_id,
                            unix_timestamp: row.unix_timestamp,
                            content: row.content,
                        })
                        .collect()
                })
            },
        }
    }

    pub async fn get_child_container_ids(&self, container_id: &str) -> Result<Vec<String>, Error> {
        sqlx::query!(
            "SELECT container_id FROM containers WHERE container_parent_id=?;",
            container_id
        )
        .fetch_all(&mut *self.connection.lock().await)
        .await
        .map(|rows| rows.into_iter().map(|row| row.container_id).collect())
    }

    /// This method only looks in the container itself, not its children.
    pub async fn get_ngram_count_in_container(&self, container_id: &str) -> Result<i32, Error> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM ngrams WHERE container_id=?;",
            container_id
        )
        .fetch_one(&mut *self.connection.lock().await)
        .await?;

        Ok(result.count)
    }

    /// This method only looks in the container itself, not its children.
    pub async fn get_entries_count_in_container(&self, container_id: &str) -> Result<i32, Error> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM entries WHERE container_id=?;",
            container_id
        )
        .fetch_one(&mut *self.connection.lock().await)
        .await?;

        Ok(result.count)
    }
}
