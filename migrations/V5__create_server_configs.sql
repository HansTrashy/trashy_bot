CREATE TABLE server_configs (
    id SERIAL8 PRIMARY KEY,
    server_id INT8 NOT NULL,
    config JSONB NOT NULL
);