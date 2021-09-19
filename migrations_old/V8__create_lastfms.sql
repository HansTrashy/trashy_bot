CREATE TABLE IF NOT EXISTS lastfms (
    id SERIAL8 PRIMARY KEY,
    server_id INT8 NOT NULL,
    user_id INT8 NOT NULL,
    username TEXT NOT NULL
);