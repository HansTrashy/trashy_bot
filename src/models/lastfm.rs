use postgres::{row::Row, Client};

pub type DbError = Box<dyn std::error::Error>;

#[derive(Debug)]
pub struct Lastfm {
    pub id: i64,
    pub server_id: i64, //TODO remove this unnecessary field
    pub user_id: i64,
    pub username: String,
}

impl Lastfm {
    pub fn get(client: &mut Client, user_id: i64) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "SELECT * FROM lastfms WHERE user_id = $1",
            &[&user_id],
        )?)?)
    }

    pub fn create(client: &mut Client, user_id: i64, username: String) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "INSERT INTO lastfms (server_id, user_id, username) VALUES (0, $1, $2) RETURNING *",
            &[&user_id, &username],
        )?)?)
    }

    pub fn update(client: &mut Client, user_id: i64, username: String) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "UPDATE lastfms SET username = $2 WHERE user_id = $1",
            &[&user_id, &username],
        )?)?)
    }

    pub fn delete(client: &mut Client, user_id: i64) -> Result<u64, DbError> {
        Ok(client.execute("DELETE FROM lastfms WHERE server_id = $1", &[&user_id])?)
    }

    fn from_row(row: Row) -> Result<Self, DbError> {
        Ok(Self {
            id: row.try_get("id")?,
            server_id: row.try_get("server_id")?,
            user_id: row.try_get("user_id")?,
            username: row.try_get("username")?,
        })
    }
}
