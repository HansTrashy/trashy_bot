CREATE TABLE banks (
    id SERIAL8 PRIMARY KEY,
    user_id INT8 NOT NULL,
    user_name TEXT NOT NULL,
    amount INT8 NOT NULL,
    last_payday TIMESTAMP NOT NULL
);