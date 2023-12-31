pub struct Message {
    pub content: String,
    pub sanitized_content: String,
    pub sender: String,
    pub room_id: String,
    pub message_id: String,
    pub unix_timestamp: i64,
}

impl Message {
    pub fn sanitize_content(content: &String) -> String {
        // TODO: Sanitize content
        content.clone()
    }

    pub fn re_sanitize(&mut self) {
        self.sanitized_content = Message::sanitize_content(&self.content);
    }
}
