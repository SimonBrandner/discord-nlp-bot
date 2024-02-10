use nlp_bot_api::{displayers::ascii_table::display_ngram_list, processor::Processor};
use serenity::all::Member;
use std::sync::Arc;

use crate::message_formatters::format_table;

pub struct SharedCommandData {
    pub processor: Arc<Processor>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, SharedCommandData, Error>;

pub async fn on_error(error: poise::FrameworkError<'_, SharedCommandData, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e);
            }
        }
    }
}

async fn send_error_message(context: &Context<'_>, error: &str) -> Result<(), Error> {
    context
        .channel_id()
        .say(
            context.http(),
            format!("Failed to get information from database! {}", error),
        )
        .await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    track_edits,
    aliases("ngrams_by_count"),
    required_permissions = "SEND_MESSAGES"
)]
pub async fn on_ngrams_by_count(
    context: Context<'_>,
    #[description = "Look for n-grams sent by this user."] sender: Option<Member>,
    #[description = "Length of the n-grams to look for."] length: Option<u32>,
    #[description = "The amount of n-grams to get."] amount: Option<u32>,
) -> Result<(), Error> {
    if length.is_some() && length < Some(1) || length > Some(5) {
        return send_error_message(
            &context,
            "The length of the n-grams must be between 1 and 5!",
        )
        .await;
    }

    let ngrams_result = context
        .data()
        .processor
        .clone()
        .get_ngrams_by_count(
            sender.clone().map(|sender| sender.user.to_string()),
            length,
            amount,
            vec![context.channel_id().to_string()].as_slice(),
        )
        .await;

    let ngrams = match ngrams_result {
        Ok(ngrams) => ngrams,
        Err(e) => {
            return send_error_message(&context, &e.to_string()).await;
        }
    };

    if ngrams.is_empty() {
        context
            .channel_id()
            .say(context.http(), "No n-grams found!")
            .await?;
        return Ok(());
    }

    let mut heading = String::from("Here's a table of n-grams by occurrence count");
    if let Some(sender) = sender {
        heading += &format!(" sent by {}", sender);
    }
    heading += ":";

    let ngrams_table = display_ngram_list(ngrams.as_slice());
    let ngrams_message_content = format_table(&ngrams_table, &heading);
    context
        .channel_id()
        .say(context.http(), ngrams_message_content)
        .await?;

    Ok(())
}
