pub mod filters;
mod utils;

use self::filters::NgramsByContentFilter;
use self::filters::NgramsByCountFilter;
use self::utils::build_in_clause;
use crate::processor::container;
use crate::processor::entry;
use crate::processor::entry::Entry;
use crate::processor::ngram::NgramsForByContentCommand;
use crate::processor::ngram::{NgramForByCountCommand, NgramForStore};
use sqlx::migrate;
use sqlx::migrate::MigrateError;
use sqlx::sqlite::SqliteQueryResult;
use sqlx::Error;
use sqlx::QueryBuilder;
use sqlx::Row;
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

    pub async fn mark_entry_as_ngrams_cached(&self, entry_ids: &[String]) -> Result<(), Error> {
        if entry_ids.is_empty() {
            return Ok(());
        }

        let mut query_builder = QueryBuilder::new("UPDATE entries SET ngrams_cached=true WHERE");
        build_in_clause(&mut query_builder, "entry_id", entry_ids);
        query_builder.push(";");

        query_builder
            .build()
            .execute(&mut *self.connection.lock().await)
            .await?;

        Ok(())
    }

    pub async fn add_ngrams(&self, ngrams: &[NgramForStore]) -> Result<(), Error> {
        if ngrams.is_empty() {
            return Ok(());
        }

        for ngrams_chunk in ngrams.chunks(CHUNK_SIZE) {
            let mut query_builder = QueryBuilder::new(
                "INSERT INTO ngrams (count, content, length, time, sender_id, container_id) ",
            );

            query_builder.push_values(ngrams_chunk, |mut query_builder, ngram| {
                query_builder
                    .push_bind(1)
                    .push_bind(ngram.content.clone())
                    .push_bind(ngram.length)
                    .push_bind(ngram.time)
                    .push_bind(ngram.sender_id.clone())
                    .push_bind(ngram.container_id.clone());
            });
            query_builder.push(
                " ON CONFLICT (content, length, time, sender_id, container_id) DO UPDATE SET count = count + 1;",
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
            query_builder.push(" ON CONFLICT (entry_id) DO NOTHING;");
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
            "INSERT INTO containers (container_id, container_parent_id) VALUES (?, ?) ON CONFLICT DO NOTHING;",
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

    pub async fn get_ngrams_by_count(
        &self,
        filter: &NgramsByCountFilter,
    ) -> Result<Vec<NgramForByCountCommand>, Error> {
        let mut query_builder =
            QueryBuilder::new("SELECT content, SUM(count) as total_count FROM ngrams");

        if !filter.container_ids.is_empty() || filter.sender_id.is_some() || filter.length.is_some()
        {
            query_builder.push(" WHERE ");
        }

        if !filter.container_ids.is_empty() {
            build_in_clause(
                &mut query_builder,
                "container_id",
                filter.container_ids.as_slice(),
            );
        }

        if !filter.container_ids.is_empty() && filter.sender_id.is_some() {
            query_builder.push(" AND ");
        }

        if let Some(sender_id) = &filter.sender_id {
            query_builder.push("sender_id=");
            query_builder.push_bind(sender_id);
        }

        if filter.sender_id.is_some() && filter.length.is_some() {
            query_builder.push(" AND ");
        }

        if let Some(length) = &filter.length {
            query_builder.push("length=");
            query_builder.push_bind(length);
        }

        query_builder.push(" GROUP BY content ORDER BY total_count ");

        match filter.order {
            filters::Order::Ascending => query_builder.push("ASC "),
            filters::Order::Descending => query_builder.push("DESC "),
        };

        query_builder.push("LIMIT ");
        query_builder.push_bind(filter.limit);

        let ngrams = query_builder
            .build()
            .fetch_all(&mut *self.connection.lock().await)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|row| NgramForByCountCommand {
                        content: row.get("content"),
                        count: row.get("total_count"),
                    })
                    .collect()
            })?;

        Ok(ngrams)
    }

    pub async fn get_ngrams_by_content(
        &self,
        filter: &NgramsByContentFilter,
    ) -> Result<Vec<NgramsForByContentCommand>, Error> {
        let mut query_builder = QueryBuilder::new("SELECT count, time FROM ngrams WHERE");

        query_builder.push(" content=");
        query_builder.push_bind(&filter.content);

        if !filter.container_ids.is_empty() {
            query_builder.push(" AND ");
            build_in_clause(
                &mut query_builder,
                "container_id",
                filter.container_ids.as_slice(),
            );
        }

        if let Some(sender_id) = &filter.sender_id {
            query_builder.push(" AND sender_id=");
            query_builder.push_bind(sender_id);
        }

        query_builder.push(" ORDER BY time ASC;");

        query_builder
            .build()
            .fetch_all(&mut *self.connection.lock().await)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|row| NgramsForByContentCommand {
                        count: row.get("count"),
                        time: row.get("time"),
                    })
                    .collect()
            })
    }
}
