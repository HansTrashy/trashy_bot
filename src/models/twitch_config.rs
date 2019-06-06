use crate::schema::twitch_configs;
use diesel::prelude::*;

#[derive(Identifiable, AsChangeset, Queryable, Debug, Clone)]
pub struct TwitchConfig {
    pub id: i64,
    pub guild_id: i64,
    pub channel_ids: Vec<i64>,
    pub delete_offline: bool,
    pub allow_everyone: bool,
}

#[derive(Insertable)]
#[table_name = "twitch_configs"]
pub struct NewTwitchConfig {
    guild_id: i64,
    channel_ids: Vec<i64>,
    delete_offline: bool,
    allow_everyone: bool,
}

pub fn create_twitch_config(
    conn: &PgConnection,
    guild_id: i64,
    channel_ids: Vec<i64>,
    delete_offline: bool,
    allow_everyone: bool,
) -> TwitchConfig {
    let new_twitch_config = NewTwitchConfig {
        guild_id,
        channel_ids,
        delete_offline,
        allow_everyone,
    };

    diesel::insert_into(twitch_configs::table)
        .values(&new_twitch_config)
        .get_result(conn)
        .expect("Error saving twitch config")
}
