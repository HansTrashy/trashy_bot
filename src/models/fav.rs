#![allow(clippy::module_name_repetitions)]
use crate::schema::favs;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Debug)]
pub struct Fav {
    pub id: i64,
    pub server_id: i64,
    pub channel_id: i64,
    pub msg_id: i64,
    pub user_id: i64,
    pub author_id: i64,
}

#[derive(Insertable)]
#[table_name = "favs"]
pub struct NewFav {
    server_id: i64,
    channel_id: i64,
    msg_id: i64,
    user_id: i64,
    author_id: i64,
}

pub fn create_fav(
    conn: &PgConnection,
    server_id: i64,
    channel_id: i64,
    msg_id: i64,
    user_id: i64,
    author_id: i64,
) -> Fav {
    let new_fav = NewFav {
        server_id,
        channel_id,
        msg_id,
        user_id,
        author_id,
    };

    diesel::insert_into(favs::table)
        .values(&new_fav)
        .get_result(conn)
        .expect("Error saving fav")
}
