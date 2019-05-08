# trashy_bot [![Build Status](https://travis-ci.com/HansTrashy/trashy_bot.svg?branch=master)](https://travis-ci.com/HansTrashy/trashy_bot)

## Requirements

Postgres installation [PostgreSQL](https://www.postgresql.org/download/) with pg/bin in PATH

*Remember to do a `cargo clean` and check that the env vars are  updated*

## Env setup

Create a `.env` file after the following example in the project root:

    DISCORD_TOKEN=****
    DATABASE_URL=postgres://{pguser}:{pgpw}:{host}


## Getting the Database up & running

Simply use `diesel database setup` for first time setup.

## Doing things with the Database

- `diesel database reset` < When you fucked up
- `diesel migration generate {create_xy}` < When you want to create a new migration
- `diesel migration run` < When you want to run your migration
- `diesel migration redo` < When your fuckup was not as big

