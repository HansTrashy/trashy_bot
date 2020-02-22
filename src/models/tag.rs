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

    pub fn belonging_to(client: &mut Client, favs: &[Fav]) -> Result<Vec<Vec<Self>>, DbError> {
        Ok(client
            .query(
                "SELECT * FROM tags WHERE fav_id = ANY($1) ORDER BY fav_id",
                &[&favs.iter().map(|f| f.id).collect::<Vec<_>>()],
            )?
            .into_iter()
            .map(Self::from_row)
            .filter_map(Result::ok)
            .group_by(|tag| tag.fav_id)
            .into_iter()
            .map(|(_key, group)| group.collect::<Vec<Self>>())
            .collect::<Vec<_>>())
    }

    fn from_row(row: Row) -> Result<Self, DbError> {
        Ok(Self {
            id: row.try_get("id")?,
            fav_id: row.try_get("fav_id")?,
            label: row.try_get("label")?,
        })
    }
}
