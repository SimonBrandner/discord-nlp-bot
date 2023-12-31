mod bot;
mod config;
mod file;

use bot::{start_bot, Bot};
use clap::Parser;
use config::read_configuration_from_file;
use nlp_bot_api::processor::Processor;
use nlp_bot_api::store::SqlStore;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(clap::Parser, Debug)]
struct CommandLineArguments {
    /// The path to the configuration file
    #[arg(short, long, default_value = "./config.json")]
    configuration_file: String,
}

#[tokio::main]
async fn main() {
    let command_line_arguments = CommandLineArguments::parse();
    let configuration = read_configuration_from_file(command_line_arguments.configuration_file);

    let store = SqlStore::new(configuration.sql_database_path);
    let processor = Arc::new(Mutex::new(Processor::new(store)));
    let bot = Bot::new(processor.clone());

    start_bot(bot, configuration.discord_token).await;
}
