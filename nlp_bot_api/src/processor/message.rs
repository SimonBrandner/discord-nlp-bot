#[derive(Debug)]
pub struct Message {
    pub message_id: String,
    pub container_id: String,
    pub sender_id: String,
    pub unix_timestamp: i64,
    pub content: String,
}
