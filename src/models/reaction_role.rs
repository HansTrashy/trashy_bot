use crate::schema::reaction_roles;
use diesel::prelude::*;

#[derive(Identifiable, AsChangeset, Queryable, Debug, Clone)]
pub struct ReactionRole {
    pub id: i64,
    pub server_id: i64,
    pub role_id: i64,
    pub role_name: String,
    pub role_group: String,
    pub emoji: String,
}

#[derive(Insertable)]
#[table_name = "reaction_roles"]
pub struct NewReactionRole {
    server_id: i64,
    role_id: i64,
    role_name: String,
    role_group: String,
    emoji: String,
}

pub fn create_reaction_role(
    conn: &PgConnection,
    server_id: i64,
    role_id: i64,
    role_name: String,
    role_group: String,
    emoji: String,
) -> ReactionRole {
    use crate::schema::reaction_roles;

    let new_rr = NewReactionRole {
        server_id,
        role_id,
        role_name,
        role_group,
        emoji,
    };

    diesel::insert_into(reaction_roles::table)
        .values(&new_rr)
        .get_result(conn)
        .expect("Error saving reaction role")
}
