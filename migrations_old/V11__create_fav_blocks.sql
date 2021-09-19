CREATE TABLE IF NOT EXISTS fav_blocks (
    id SERIAL8 PRIMARY KEY,
    server_id INT8 NOT NULL, -- Server on which this fav is blocked
    channel_id INT8 NOT NULL, -- Channel id of the blocked fav
    msg_id INT8 NOT NULL -- Message id of the blocked fav
);