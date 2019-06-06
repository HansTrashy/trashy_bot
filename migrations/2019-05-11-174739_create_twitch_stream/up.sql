CREATE TABLE twitch_streams (
    id SERIAL8 PRIMARY KEY,
    twitch_user_id TEXT NOT NULL,
    profile_image_url TEXT NOT NULL
)