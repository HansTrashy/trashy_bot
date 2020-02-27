use crate::models::fav::Fav;
use itertools::Itertools;
use postgres::{row::Row, Client};

pub type DbError = Box<dyn std::error::Error>;

#[derive(Debug)]
pub struct Tag {
    pub id: i64,
    pub fav_id: i64,
    pub label: String,
}

impl Tag {
    pub fn create(client: &mut Client, fav_id: i64, label: &str) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "INSERT INTO tags (fav_id, label) VALUES ($1, $2) RETURNING *",
            &[&fav_id, &label],
        )?)?)
    }

    pub fn delete(client: &mut Client, fav_id: i64) -> Result<u64, DbError> {
        Ok(client.execute("DELETE FROM tags WHERE fav_id = $1", &[&fav_id])?)
    }

    pub fn of_user(client: &mut Client, user_id: i64) -> Result<Vec<Self>, DbError> {
        Ok(client.query("SELECT tags.id, tags.fav_id, tags.label FROM tags INNER JOIN favs ON tags.fav_id = favs.id WHERE favs.user_id = $1",
            &[&user_id])?.into_iter().map(Self::from_row).collect::<Result<Vec<_>, DbError>>()?)
    }

    fn from_row(row: Row) -> Result<Self, DbError> {
        Ok(Self {
            id: row.try_get("id")?,
            fav_id: row.try_get("fav_id")?,
            label: row.try_get("label")?,
        })
    }
}
