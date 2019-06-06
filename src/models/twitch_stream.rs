use crate::schema::twitch_streams;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Debug)]
pub struct TwitchStream {
    pub id: i64,
    pub twitch_user_id: String,
    pub profile_image_url: String,
}

#[derive(Insertable)]
#[table_name = "twitch_streams"]
pub struct NewTwitchStream {
    twitch_user_id: String,
    profile_image_url: String,
}

pub fn create_twitch_stream(
    conn: &PgConnection,
    twitch_user_id: String,
    profile_image_url: String,
) -> TwitchStream {
    let new_twitch_stream = NewTwitchStream {
        twitch_user_id,
        profile_image_url,
    };

    diesel::insert_into(twitch_streams::table)
        .values(&new_twitch_stream)
        .get_result(conn)
        .expect("Error saving TwitchStream")
}
