# bot.io

Discord bot that has you play a game: understand what a black box function does.

## Deployment notes

Most knobs are present in the `config.toml` file, which is committed in this repository.
Secrets are stored in environment variables. `.env` files are also loaded, which is useful
for development.

You will need a bot token. Please consult Discord documentation on how to obtain one.
Put the token in the `DISCORD_TOKEN` variable.

The bot uses a PostgreSQL database to store its state. The `DATABASE_URL` variable
should convey how to connect to it, in the format
```
DATABASE_URL=postgres://username:password@host/dbname
```

You will probably need to create a user and database for that
```
$ sudo -u postgres createuser -P iobot # will prompt for password
$ sudo -u postgres createdb iobot
```

To initialize the database, run
```
cargo install diesel_cli
diesel database setup
```

To later update the schema, run
```
diesel migration run
```
