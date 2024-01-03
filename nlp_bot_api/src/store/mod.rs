use crate::processor::message;
use sqlx::{migrate::MigrateDatabase, Connection, Sqlite, SqliteConnection};
use tokio::sync::Mutex;

pub struct SqlStore {
    connection: Mutex<SqliteConnection>,
}

impl SqlStore {
    pub async fn new(file_path: String) -> Result<Self, sqlx::Error> {
        if !Sqlite::database_exists(&file_path).await.unwrap_or(false) {
            log::info!("Database does not exist - creating...");
            match Sqlite::create_database(&file_path).await {
                Ok(_) => log::info!("Database created successfully!"),
                Err(e) => return Err(e),
            }
        }

        match SqliteConnection::connect(format!("sqlite://{}", file_path).as_str()).await {
            Ok(c) => Ok(Self {
                connection: Mutex::new(c),
            }),
            Err(e) => Err(e),
        }
    }

    pub async fn add_message(&self, message: message::Message) {
        sqlx::query!(
            "INSERT INTO messages (message_id, content, sanitized_content, sender_id, container_id, unix_timestamp) VALUES (?, ?, ?, ?, ?, ?);",
            message.message_id,
            message.content,
            message.sanitized_content,
            message.sender_id,
            message.container_id,
            message.unix_timestamp,
        ).execute(&mut *self.connection.lock().await).await.expect("Failed to write to database");
    }
}
