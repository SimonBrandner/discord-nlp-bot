use crate::processor::container;
use crate::processor::message;
use sqlx::migrate;
use sqlx::migrate::MigrateError;
use sqlx::Error;
use sqlx::{migrate::MigrateDatabase, Connection, Sqlite, SqliteConnection};
use tokio::sync::Mutex;

pub struct Sql {
    connection: Mutex<SqliteConnection>,
}

impl Sql {
    pub async fn new(file_path: String) -> Result<Self, sqlx::Error> {
        if !Sqlite::database_exists(&file_path).await.unwrap_or(false) {
            log::info!("Database does not exist - creating...");
            match Sqlite::create_database(&file_path).await {
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

    pub async fn add_message(&self, message: message::Message) {
        sqlx::query!(
            "INSERT INTO entries (message_id, content, sender_id, container_id, unix_timestamp) VALUES (?, ?, ?, ?, ?);",
            message.message_id,
            message.content,
            message.sender_id,
            message.container_id,
            message.unix_timestamp,
        ).execute(&mut *self.connection.lock().await).await.expect("Failed to add message to database!");
    }

    pub async fn add_container(&self, container: container::Container) {
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
}
