use serenity::model::channel;

const NO_GUILD_STRING: &str = "no_guild";

pub struct Message {
    content: String,
    sanitized_content: String,
    sender: String,
    room_id: String,
}

impl Message {
    pub fn sanitize_content(content: &String) -> String {
        // TODO: Sanitize content
        content.clone()
    }

    pub fn from_discord(discord_message: &channel::Message) -> Self {
        Self {
            content: discord_message.content.clone(),
            sanitized_content: Message::sanitize_content(&discord_message.content),
            sender: discord_message.author.to_string(),
            room_id: format!(
                "{}:{}",
                match discord_message.guild_id {
                    Some(id) => id.to_string(),
                    None => String::from(NO_GUILD_STRING),
                },
                discord_message.channel_id.to_string()
            ),
        }
    }

    pub fn re_sanitize(&mut self) {
        self.sanitized_content = Message::sanitize_content(&self.content);
    }
}
