CREATE TABLE twitch_configs (
    id SERIAL8 PRIMARY KEY,
    guild_id INT8 NOT NULL,
    channel_ids INT8[] NOT NULL,
    delete_offline BOOLEAN NOT NULL,
    allow_everyone BOOLEAN NOT NULL
)