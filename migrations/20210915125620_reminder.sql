CREATE TABLE IF NOT EXISTS reminders (
    id SERIAL8 PRIMARY KEY,
    channel_id INT8 NOT NULL,
    user_id INT8 NOT NULL,
    anchor_id INT8 NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    msg TEXT NOT NULL
);