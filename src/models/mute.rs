use chrono::{DateTime, Utc};
use postgres::{row::Row, Client};

pub type DbError = Box<dyn std::error::Error>;

#[derive(Debug)]
pub struct Mute {
    pub id: i64,
    pub server_id: i64,
    pub user_id: i64,
    pub end_time: DateTime<Utc>,
}

impl Mute {
    pub fn get(client: &mut Client, server_id: i64, user_id: i64) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "SELECT * FROM mutes WHERE user_id = $1 AND server_id = $2",
            &[&user_id, &server_id],
        )?)?)
    }

    pub fn create(
        client: &mut Client,
        server_id: i64,
        user_id: i64,
        end_time: DateTime<Utc>,
    ) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "INSERT INTO mutes VALUES (server_id, user_id, end_time) = ($1, $2, $3) RETURNING *",
            &[&server_id, &user_id, &end_time],
        )?)?)
    }

    pub fn update(
        client: &mut Client,
        server_id: i64,
        user_id: i64,
        end_time: DateTime<Utc>,
    ) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "UPDATE mutes SET end_time = $3 WHERE server_id = $1 AND user_id = $1",
            &[&server_id, &user_id, &end_time],
        )?)?)
    }

    pub fn delete(client: &mut Client, server_id: i64, user_id: i64) -> Result<u64, DbError> {
        Ok(client.execute(
            "DELETE FROM mutes WHERE server_id = $1 AND user_id = $2",
            &[&server_id, &user_id],
        )?)
    }

    fn from_row(row: Row) -> Result<Self, DbError> {
        Ok(Self {
            id: row.try_get("id")?,
            server_id: row.try_get("server_id")?,
            user_id: row.try_get("user_id")?,
            end_time: row.try_get("end_time")?,
        })
    }
}
