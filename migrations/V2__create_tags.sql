CREATE TABLE tags (
    id SERIAL8 PRIMARY KEY,
    fav_id INT8 NOT NULL,
    label TEXT NOT NULL
);