use crate::schema::reaction_roles;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Identifiable, AsChangeset, Queryable, Debug, Clone)]
#[table_name = "reaction_roles"]
pub struct ReactionRole {
    pub id: i64,
    pub server_id: i64,
    pub role_id: i64,
    pub emoji: String,
}

#[derive(Insertable)]
#[table_name = "reaction_roles"]
pub struct NewReactionRole {
    server_id: i64,
    role_id: i64,
    emoji: String,
}

pub fn create_reaction_role(
    conn: &PgConnection,
    server_id: i64,
    role_id: i64,
    emoji: String,
) -> ReactionRole {
    use crate::schema::reaction_roles;

    let new_rr = NewReactionRole {
        server_id,
        role_id,
        emoji,
    };

    diesel::insert_into(reaction_roles::table)
        .values(&new_rr)
        .get_result(conn)
        .expect("Error saving reaction role")
}
