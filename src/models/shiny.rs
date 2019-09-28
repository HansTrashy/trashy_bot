use crate::schema::shinys;
use chrono::{DateTime, Utc};

#[derive(Identifiable, AsChangeset, Queryable, Debug)]
pub struct Shiny {
    pub id: i64,
    pub server_id: i64,
    pub user_id: i64,
    pub username: String,
    pub amount: i64,
}

#[derive(Insertable, Debug)]
#[table_name = "shinys"]
pub struct NewShiny {
    pub server_id: i64,
    pub user_id: i64,
    pub username: String,
    pub amount: i64,
}
