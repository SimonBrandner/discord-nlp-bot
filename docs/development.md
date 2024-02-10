# Development

## Setup

- Install Git, Rust, SQLite, SQLX CLI
- Run the following:

```console
git clone https://github.com/SimonBrandner/discord-nlp-bot.git
cd discord-nlp-bot

cp .env.sample .env
cp discord_nlp_bot/config.sample.json discord_nlp_bot/config.json

cargo sqlx database create
cargo sqlx migrate run --source nlp_bot_api/src/migrations

cd discord_nlp_bot
cargo run
```

## Database

### Updating

- Add a new migration using `sqlx migrate add <name> --source nlp_bot_api/src/migrations`
- Run the migration using `cargo sqlx migrate run --source nlp_bot_api/src/migrations`

### Writing queries

- Run `cargo sqlx prepare --workspace`
