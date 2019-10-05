use crate::schema::lastfms;
use chrono::{DateTime, Utc};

#[derive(Identifiable, AsChangeset, Queryable, Debug)]
pub struct Lastfm {
    pub id: i64,
    pub server_id: i64,
    pub user_id: i64,
    pub username: String,
}

#[derive(Insertable, Debug)]
#[table_name = "lastfms"]
pub struct NewLastfm {
    pub server_id: i64,
    pub user_id: i64,
    pub username: String,
}
