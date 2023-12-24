mod config;
mod controller;
mod discord;
mod file;
mod store;

use clap::Parser;

#[derive(clap::Parser, Debug)]
struct CommandLineArguments {
    /// The path to the configuration file
    #[arg(short, long, default_value = "./config.json")]
    configuration_file: String,
}

#[tokio::main]
async fn main() {
    let command_line_arguments = CommandLineArguments::parse();
    let configuration =
        config::read_configuration_from_file(command_line_arguments.configuration_file);

    let sql_store = store::sql::SqlStore::new();
    let client = discord::client::DiscordClient::new(configuration.discord_token);
    let controller = controller::Controller::new(sql_store, client);
    controller.update().await;
}
