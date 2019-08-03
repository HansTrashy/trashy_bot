use crate::schema::server_configs;

#[derive(Identifiable, AsChangeset, Queryable, Debug)]
pub struct ServerConfig {
    pub id: i64,
    pub server_id: i64,
    pub config: serde_json::Value,
}

#[derive(Insertable, Debug)]
#[table_name = "server_configs"]
pub struct NewServerConfig {
    pub server_id: i64,
    pub config: serde_json::Value,
}
