#!bash
rm discord_nlp_bot/database.db
sqlite3 discord_nlp_bot/database.db < nlp_bot_api/src/schemas/database.schema
