# Discord NLP Bot

A Discord Bot to perform Natural Language Processing (NLP).

## Development

### Setup

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

### Updating database

- Add a new migration to `nlp_bot_api/src/migrations`
- Run `cargo sqlx prepare --workspace`
