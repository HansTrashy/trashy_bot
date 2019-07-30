CREATE TABLE twitch_configs (
    id SERIAL8 PRIMARY KEY,
    guild_id INT8 NOT NULL,
    channel_ids INT8[] NOT NULL,
    delete_offline BOOLEAN NOT NULL,
    allow_everyone BOOLEAN NOT NULL
);

CREATE TABLE twitch_streams (
    id SERIAL8 PRIMARY KEY,
    twitch_user_id TEXT NOT NULL,
    profile_image_url TEXT NOT NULL
);

CREATE TABLE twitch_subs (
    id SERIAL8 PRIMARY KEY,
    twitch_stream_id INT8 NOT NULL REFERENCES twitch_streams(id),
    channel_id INT8 NOT NULL,
    user_id INT8 NOT NULL,
    message_id INT8
);