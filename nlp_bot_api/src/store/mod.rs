use crate::processor::container;
use crate::processor::message;
use crate::processor::message::Message;
use crate::processor::ngram::Ngram;
use sqlx::migrate;
use sqlx::migrate::MigrateError;
use sqlx::Error;
use sqlx::QueryBuilder;
use sqlx::{migrate::MigrateDatabase, Connection, Sqlite, SqliteConnection};
use tokio::sync::Mutex;

pub struct Sql {
    connection: Mutex<SqliteConnection>,
}

impl Sql {
    pub async fn new(file_path: &str) -> Result<Self, sqlx::Error> {
        if !Sqlite::database_exists(file_path).await.unwrap_or(false) {
            log::info!("Database does not exist - creating...");
            match Sqlite::create_database(file_path).await {
                Ok(()) => log::info!("Database created successfully!"),
                Err(e) => return Err(e),
            }
        }

        let database_connection =
            match SqliteConnection::connect(format!("sqlite://{}", file_path).as_str()).await {
                Ok(c) => c,
                Err(e) => return Err(e),
            };

        let sql = Self {
            connection: Mutex::new(database_connection),
        };

        if let Err(e) = sql.migrate().await {
            return Err(e.into());
        }

        Ok(sql)
    }

    async fn migrate(&self) -> Result<(), MigrateError> {
        migrate!("./src/migrations")
            .run(&mut *self.connection.lock().await)
            .await
    }

    // TODO: Batch this
    pub async fn mark_message_ngrams_cached(&self, message_ids: &[String]) {
        let message_ids_string = message_ids.join(",");

        sqlx::query!(
            "UPDATE entries SET ngrams_cached=true WHERE message_id IN (?);",
            message_ids_string
        )
        .execute(&mut *self.connection.lock().await)
        .await
        .expect("Failed to mark message ngrams as cached!");
    }

    pub async fn add_ngrams(&self, ngrams: &[Ngram]) {
        if ngrams.is_empty() {
            return;
        }

        let mut query_builder = QueryBuilder::new(
            "INSERT INTO ngrams (count, content, time, sender_id, container_id) ",
        );
        query_builder.push_values(ngrams, |mut query_builder, ngram| {
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
            .await
            .expect("Failed to add ngrams to database!");
    }

    pub async fn add_messages(&self, messages: &[message::Message], ngrams_cached: bool) {
        if messages.is_empty() {
            return;
        }

        let mut query_builder = QueryBuilder::new(
            "INSERT INTO entries (message_id, content, sender_id, container_id, unix_timestamp, ngrams_cached) ",
        );
        query_builder.push_values(messages, |mut query_builder, message| {
            query_builder
                .push_bind(message.message_id.clone())
                .push_bind(message.content.clone())
                .push_bind(message.sender_id.clone())
                .push_bind(message.container_id.clone())
                .push_bind(message.unix_timestamp)
                .push_bind(ngrams_cached);
        });
        query_builder
            .build()
            .execute(&mut *self.connection.lock().await)
            .await
            .expect("Failed to add messages to database!");
    }

    pub async fn add_container(&self, container: &container::Container) {
        sqlx::query!(
            "INSERT INTO containers (container_id, container_parent_id) VALUES (?, ?);",
            container.container_id,
            container.container_parent_id
        )
        .execute(&mut *self.connection.lock().await)
        .await
        .expect("Failed to add container to database!");
    }

    pub async fn get_last_message_id_in_container(
        &self,
        container_id: &str,
    ) -> Result<String, Error> {
        match sqlx::query!(
            "SELECT message_id FROM entries WHERE container_id=? ORDER BY unix_timestamp DESC LIMIT 1;",
            container_id
        )
        .fetch_one(&mut *self.connection.lock().await).await {
            Ok(o) => Ok(o.message_id),
            Err(e) => Err(e),
        }
    }

    pub async fn get_first_message_id_in_container(
        &self,
        container_id: &str,
    ) -> Result<String, Error> {
        match sqlx::query!(
            "SELECT message_id FROM entries WHERE container_id=? ORDER BY unix_timestamp ASC LIMIT 1;",
            container_id
        )
        .fetch_one(&mut *self.connection.lock().await).await {
            Ok(o) => Ok(o.message_id),
            Err(e) => Err(e),
        }
    }

    pub async fn get_messages_without_cached_ngrams(
        &self,
        limit: u32,
        start_id: Option<String>,
    ) -> Result<Vec<Message>, Error> {
        match start_id {
            Some(start_id) => {
                sqlx::query!(
                    "
                    SELECT * FROM entries WHERE ngrams_cached=false AND unix_timestamp < (
                        SELECT unix_timestamp FROM entries WHERE message_id=?
                    ) ORDER BY unix_timestamp DESC LIMIT ?;
                    ",
                    start_id,
                    limit
                )
                .fetch_all(&mut *self.connection.lock().await)
                .await
                .map(|rows| {
                    rows.into_iter()
                        .map(|row| Message {
                            message_id: row.message_id,
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
                        .map(|row| Message {
                            message_id: row.message_id,
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
        match sqlx::query!(
            "SELECT COUNT(*) as count FROM ngrams WHERE container_id=?;",
            container_id
        )
        .fetch_one(&mut *self.connection.lock().await)
        .await
        {
            Ok(o) => Ok(o.count),
            Err(e) => Err(e),
        }
    }

    /// This method only looks in the container itself, not its children.
    pub async fn get_entries_count_in_container(&self, container_id: &str) -> Result<i32, Error> {
        match sqlx::query!(
            "SELECT COUNT(*) as count FROM entries WHERE container_id=?;",
            container_id
        )
        .fetch_one(&mut *self.connection.lock().await)
        .await
        {
            Ok(o) => Ok(o.count),
            Err(e) => Err(e),
        }
    }
}
