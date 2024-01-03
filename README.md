# Discord NLP Bot

A Discord Bot to perform Natural Language Processing (NLP).

## Development setup

- Install Git, Rust, SQLite
- Run the following:

```console
git clone https://github.com/SimonBrandner/discord-nlp-bot.git
cd discord-nlp-bot
sqlite3 discord_nlp_bot/database.db < nlp_bot_api/src/schemas/database.schema
echo "DATABASE_URL=sqlite://./discord_nlp_bot/database.db" > .env
cp discord_nlp_bot/config.sample.json discord_nlp_bot/config.json
```

- Have fun
