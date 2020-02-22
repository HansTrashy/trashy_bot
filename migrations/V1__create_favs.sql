CREATE TABLE IF NOT EXISTS favs (
    id SERIAL8 PRIMARY KEY,
    server_id INT8 NOT NULL,
    channel_id INT8 NOT NULL,
    msg_id INT8 NOT NULL,
    user_id INT8 NOT NULL,
    author_id INT8 NOT NULL
);