use crate::schema::mutes;
use chrono::{DateTime, Utc};

#[derive(Identifiable, AsChangeset, Queryable, Debug)]
pub struct Mute {
    pub id: i64,
    pub server_id: i64,
    pub user_id: i64,
    pub end_time: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[table_name = "mutes"]
pub struct NewMute {
    pub server_id: i64,
    pub user_id: i64,
    pub end_time: DateTime<Utc>,
}
