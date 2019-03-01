use crate::schema::favs;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Debug)]
pub struct Fav {
    id: i64,
    server_id: i64,
    channel_id: i64,
    msg_id: i64,
    user_id: i64,
    author_id: i64,
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
    use crate::schema::favs;

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
