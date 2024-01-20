#[derive(Debug)]
pub struct Message {
    pub message_id: String,
    pub container_id: String,
    pub sender_id: String,
    pub unix_timestamp: i64,
    pub content: String,
    pub sanitized_content: String,
}

impl Message {
    pub fn sanitize_content(content: &str) -> String {
        // TODO: Sanitize content
        content.to_string()
    }

    pub fn re_sanitize(&mut self) {
        self.sanitized_content = Self::sanitize_content(&self.content);
    }
}
