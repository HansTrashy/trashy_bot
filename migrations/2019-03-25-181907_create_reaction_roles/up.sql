CREATE TABLE reaction_roles (
    id SERIAL8 PRIMARY KEY,
    server_id INT8 NOT NULL,
    role_id INT8 NOT NULL,
    role_name TEXT NOT NULL,
    role_group TEXT NOT NULL,
    emoji TEXT NOT NULL
)