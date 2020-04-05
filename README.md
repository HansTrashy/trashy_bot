# trashy_bot [![Build Status](https://travis-ci.com/HansTrashy/trashy_bot.svg?branch=master)](https://travis-ci.com/HansTrashy/trashy_bot)

## Requirements

No external dependencies (at least until voice is implemented again?)

*Remember to do a `cargo clean` and check that the env vars are  updated*

## Env setup

Create a `.env` file after the following example in the project root:

    DISCORD_TOKEN=****
    TWITCH_TOKEN=****
    PG_HOST=localhost
    PG_USER=*user*
    PG_PASSWORD=*pw*
    PG_DBNAME=trashy_bot
    PG_POOL.MAX_SIZE=4
    PG_POOL.TIMEOUTS.WAIT.SECS=5
    PG_POOL.TIMEOUTS.WAIT.NANOS=0

