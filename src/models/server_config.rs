use crate::schema::server_configs;
use crate::schema::server_settings;

#[derive(Identifiable, Queryable, Debug)]
pub struct ServerConfig {
    id: i32,
    server_id: i32,
}

#[derive(Insertable, Debug)]
#[table_name = "server_configs"]
pub struct NewServerConfig {
    server_id: i32,
}

#[derive(Identifiable, Queryable, Debug, Associations)]
#[belongs_to(ServerConfig)]
pub struct ServerSetting {
    id: i32,
    server_config_id: i32,
    key: String,
    value: String,
}

#[derive(Insertable, Debug)]
#[table_name = "server_settings"]
pub struct NewServerSetting {
    server_config_id: i32,
    key: String,
    value: String,
}
