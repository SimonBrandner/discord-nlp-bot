use crate::processor::message;

use sqlite::Connection;

pub struct SqlStore {
    connection: Connection,
}

impl SqlStore {
    pub fn new(file_path: String) -> Self {
        Self {
            connection: sqlite::open(file_path).unwrap(),
        }
    }

    pub fn add_message(&self, message: message::Message) {
        self.connection.execute(format!(
            "INSERT INTO messages (message_id, sanitized_content, sender, room_id, unix_timestamp) VALUE ({}, {}, {}, {}, {})",
            message.message_id,
            message.sanitized_content, 
            message.sender, 
            message.container_id,
            message.unix_timestamp
        )).expect("Failed to write to database");
    }
}
