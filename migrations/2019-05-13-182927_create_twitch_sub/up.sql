CREATE TABLE twitch_subs (
    id SERIAL8 PRIMARY KEY,
    twitch_stream_id INT8 NOT NULL REFERENCES twitch_streams(id),
    channel_id INT8 NOT NULL,
    user_id INT8 NOT NULL,
    message_id INT8
)