use std::sync::Mutex;

use crate::processor::message;

use sqlite::Connection;

pub struct SqlStore {
    connection: Mutex<Connection>,
}

impl SqlStore {
    pub fn new(file_path: String) -> Self {
        Self {
            connection: Mutex::new(sqlite::open(file_path).unwrap()),
        }
    }

    pub fn add_message(&self, message: message::Message) {
        // TODO: Does this Mutex thing really work
        self.connection.lock().unwrap().execute(format!(
            "INSERT INTO messages (message_id, sanitized_content, sender, room_id, unix_timestamp) VALUE ({}, {}, {}, {}, {})",
            message.message_id,
            message.sanitized_content, 
            message.sender, 
            message.room_id,
            message.unix_timestamp
        )).expect("Failed to write to database");
    }
}
