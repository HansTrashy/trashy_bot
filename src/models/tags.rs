use crate::models::favs::Fav;
use crate::schema::tags;
use diesel::prelude::*;

#[derive(Identifiable, Queryable, Associations, Debug)]
#[belongs_to(Fav)]
pub struct Tag {
        id: i64,
        fav_id: i64,
        label: String,
}

#[derive(Insertable)]
#[table_name = "tags"]
pub struct NewTag {
        fav_id: i64,
        label: String,
}

pub fn create_tag(conn: &PgConnection, fav_id: i64, label: String) -> Tag {
        use crate::schema::tags;

        let new_tag = NewTag { fav_id, label };

        diesel::insert_into(tags::table)
                .values(&new_tag)
                .get_result(conn)
                .expect("Error saving tag")
}

pub fn create_tags(conn: &PgConnection, new_tags: Vec<NewTag>) -> Vec<Tag> {
        use crate::schema::tags;

        diesel::insert_into(tags::table)
                .values(&new_tags)
                .get_results(conn)
                .expect("Error saving tag")
}
