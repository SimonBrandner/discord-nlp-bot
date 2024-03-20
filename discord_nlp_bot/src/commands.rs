use nlp_bot_api::displayers::chart::display_ngram_count_over_time;
use nlp_bot_api::{
    displayers::ascii_table::display_ngram_list, processor::Processor, store::filters::Order,
};
use poise::CreateReply;
use serenity::all::Member;
use serenity::builder::CreateAttachment;
use std::cmp::Ordering;
use std::str::FromStr;
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
        .say(format!(
            "Failed to get information from database! {}",
            error
        ))
        .await?;

    Ok(())
}

fn get_container_ids_from_context(
    context: &Context<'_>,
    container_context: &Option<String>,
) -> Result<Vec<String>, String> {
    let container_ids: Vec<String>;
    match container_context.as_deref().unwrap_or("server") {
        "channel" => container_ids = vec![context.channel_id().to_string()],
        "server" => match context.guild_id() {
            Some(guild_id) => container_ids = vec![guild_id.to_string()],
            None => return Err("You can't use the `server` container in a DM!".into()),
        },
        "discord" => container_ids = vec!["discord".to_string()],
        "all" => container_ids = vec![],
        _ => {
            return Err(
                "The container you specified was neither `channel`, `server`, `discord` nor `all`"
                    .into(),
            );
        }
    };

    Ok(container_ids)
}

fn get_options_text(mut options: Vec<Option<(&str, String, bool)>>) -> String {
    options.sort();
    let filtered_options: Vec<(&str, String, bool)> =
        options.iter().filter_map(|o| o.as_ref().cloned()).collect();

    let mut text = String::new();
    for (index, (name, description, code_block)) in filtered_options.iter().enumerate() {
        if *code_block {
            text += &format!("{} `{}`", name, description);
        } else {
            text += &format!("{} {}", name, description);
        }

        if filtered_options.len() < 2 {
            continue;
        }

        match index.cmp(&(filtered_options.len() - 2)) {
            Ordering::Less => text += ", ",
            Ordering::Equal => text += " and ",
            Ordering::Greater => (),
        }
    }
    text
}

#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    required_permissions = "SEND_MESSAGES"
)]
pub async fn ngrams_by_count(
    context: Context<'_>,
    #[description = "Look for n-grams sent by this user."] sender: Option<Member>,
    #[description = "Length of the n-grams to look for."] length: Option<u32>,
    #[description = "The amount of n-grams to get."] amount: Option<u32>,
    #[description = "Look for n-grams sent in this context. Either `channel`, `server`, `discord` or `all`."]
    #[rename = "context"]
    container_context: Option<String>,
    #[description = "The way to order n-grams by occurrence count. Either `asc` or `desc`."]
    #[rename = "order"]
    order_string: Option<String>,
) -> Result<(), Error> {
    // This command can take some time
    if let Err(error) = context.defer().await {
        return send_error_message(&context, &error.to_string()).await;
    };

    let mut order: Option<Order> = None;
    if let Some(order_string) = &order_string {
        match Order::from_str(order_string) {
            Ok(order_string) => order = Some(order_string),
            Err(_) => {
                return send_error_message(
                    &context,
                    "The order you specified was neither `asc` nor `desc`",
                )
                .await;
            }
        }
    }
    let container_ids = match get_container_ids_from_context(&context, &container_context) {
        Ok(container_ids) => container_ids,
        Err(error) => return send_error_message(&context, &error).await,
    };
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
            &container_ids,
            order.clone(),
        )
        .await;

    let ngrams = match ngrams_result {
        Ok(ngrams) => ngrams,
        Err(e) => {
            return send_error_message(&context, &e.to_string()).await;
        }
    };

    if ngrams.is_empty() {
        context.say("No n-grams found!").await?;
        return Ok(());
    }

    let heading = format!(
        "Here's a table of n-grams by occurrence count with {}",
        get_options_text(vec![
            sender.map(|s| ("sender", s.user.to_string(), false)),
            container_context.map(|c| ("context", c, true)),
            order_string.map(|o| ("order", o, true))
        ])
    );

    let ngrams_table = display_ngram_list(ngrams.as_slice());
    let ngrams_message_content = format_table(&ngrams_table, &heading);
    context.say(ngrams_message_content).await?;

    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    required_permissions = "SEND_MESSAGES"
)]
pub async fn ngram_by_content(
    context: Context<'_>,
    #[rename = "ngram"]
    #[description = "The ngram about which to get data."]
    ngram_content: String,
    #[rename = "sender"]
    #[description = "Look for n-grams sent by this user."]
    sender: Option<Member>,
    #[rename = "context"]
    #[description = "Look for n-grams sent in this context. Either `channel`, `server`, `discord` or `all`."]
    container_context: Option<String>,
) -> Result<(), Error> {
    // This command can take some time
    if let Err(error) = context.defer().await {
        return send_error_message(&context, &error.to_string()).await;
    };

    let processor = context.data().processor.clone();
    let container_ids = match get_container_ids_from_context(&context, &container_context) {
        Ok(container_ids) => container_ids,
        Err(error) => return send_error_message(&context, &error).await,
    };

    let ngrams_result = processor
        .get_ngram_by_content(
            &ngram_content,
            sender.clone().map(|sender| sender.user.to_string()),
            &container_ids,
        )
        .await;
    let ngrams = match ngrams_result {
        Ok(ngrams) => ngrams,
        Err(e) => {
            return send_error_message(&context, &e.to_string()).await;
        }
    };
    if ngrams.is_empty() {
        context.say("No n-grams found!").await?;
        return Ok(());
    }
    let image = match display_ngram_count_over_time(&ngrams) {
        Ok(image) => image,
        Err(e) => {
            return send_error_message(&context, &e.to_string()).await;
        }
    };
    let message = format!(
        "Here's a chart of the number of occurrences of the n-gram `{}` over time with {}",
        ngram_content,
        get_options_text(vec![
            sender.map(|s| ("sender", s.user.to_string(), false)),
            container_context.map(|c| ("context", c, true)),
        ])
    );

    context
        .send(
            CreateReply::default()
                .content(message)
                .attachment(CreateAttachment::bytes(image, "chart.png")),
        )
        .await?;

    Ok(())
}
