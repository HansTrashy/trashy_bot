CREATE TABLE server_configs (
    id SERIAL PRIMARY KEY,
    server_id INT NOT NULL
);

CREATE TABLE server_settings (
    id SERIAL PRIMARY KEY,
    server_config_id INT NOT NULL REFERENCES server_configs(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value TEXT NOT NULL
);