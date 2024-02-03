mod bot;
mod config;
mod file;
mod makers;

use bot::{start, Bot};
use clap::Parser;
use config::read_configuration_from_file;
use nlp_bot_api::processor::Processor;
use nlp_bot_api::store::Sql;
use std::sync::Arc;

#[derive(clap::Parser, Debug)]
struct CommandLineArguments {
    /// The path to the configuration file
    #[arg(short, long, default_value = "./config.json")]
    configuration_file: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let command_line_arguments = CommandLineArguments::parse();
    let configuration = read_configuration_from_file(&command_line_arguments.configuration_file);

    let store = match Sql::new(&configuration.sql_database_path).await {
        Ok(store) => store,
        Err(e) => panic!("Failed to construct store: {}", e),
    };

    let processor = Arc::new(Processor::new(store));
    let bot = Bot::new(processor.clone());

    log::info!("Starting bot...");
    tokio::spawn(async move { processor.clone().cache_ngrams().await });
    start(bot, configuration.discord_token).await;
}
