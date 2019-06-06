use crate::models::twitch_stream::TwitchStream;
use crate::schema::twitch_subs;
use diesel::prelude::*;

#[derive(Identifiable, Associations, Queryable, Debug)]
#[belongs_to(TwitchStream)]
pub struct TwitchSub {
    pub id: i64,
    pub twitch_stream_id: i64,
    pub channel_id: i64,
    pub user_id: i64,
    pub message_id: Option<i64>,
}

#[derive(Insertable)]
#[table_name = "twitch_subs"]
pub struct NewTwitchSub {
    twitch_stream_id: i64,
    channel_id: i64,
    user_id: i64,
    message_id: Option<i64>,
}

pub fn create_twitch_sub(
    conn: &PgConnection,
    twitch_stream_id: i64,
    channel_id: i64,
    user_id: i64,
    message_id: Option<i64>,
) -> TwitchSub {
    let new_twitch_sub = NewTwitchSub {
        twitch_stream_id,
        channel_id,
        user_id,
        message_id,
    };

    diesel::insert_into(twitch_subs::table)
        .values(&new_twitch_sub)
        .get_result(conn)
        .expect("Error saving TwitchSub")
}
