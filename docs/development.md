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

## NixOS

You will need to have flakes enabled (see the [NixOS Wiki](https://nixos.wiki/wiki/Flakes)).

### The development environment

You can use the following command to enter the development environment.

```console
nix develop
```

### VS Code

If you want the `rust-analyzer` VS Code extension to work correctly, you'll need
to have an `.envrc` file in your directory and the
[`direnv`](https://marketplace.visualstudio.com/items?itemName=mkhl.direnv)
extension installed.

```console
cp .envrc.sample .envrc
```

## Database

### Updating

- Add a new migration using `sqlx migrate add <name> --source nlp_bot_api/src/migrations`
- Run the migration using `cargo sqlx migrate run --source nlp_bot_api/src/migrations`

### Writing queries

- Run `cargo sqlx prepare --workspace`
