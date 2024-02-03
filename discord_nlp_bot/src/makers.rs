use nlp_bot_api::processor::entry::Entry;
use serenity::all::Message;

pub fn make_entry(discord_message: &Message) -> Entry {
    Entry {
        content: discord_message.content.clone(),
        container_id: discord_message.channel_id.to_string(),
        sender_id: discord_message.author.to_string(),
        unix_timestamp: discord_message.timestamp.unix_timestamp(),
        entry_id: discord_message.id.to_string(),
    }
}
