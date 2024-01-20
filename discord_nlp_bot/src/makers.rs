use nlp_bot_api::processor;

pub fn make_message(discord_message: &serenity::all::Message) -> processor::message::Message {
    processor::message::Message {
        content: discord_message.content.clone(),
        sanitized_content: processor::message::Message::sanitize_content(&discord_message.content),
        container_id: discord_message.channel_id.to_string(),
        sender_id: discord_message.author.to_string(),
        unix_timestamp: discord_message.timestamp.unix_timestamp(),
        message_id: discord_message.id.to_string(),
    }
}
