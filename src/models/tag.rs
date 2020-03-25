use tokio_postgres::{row::Row, Client};

pub type DbError = String;

#[derive(Debug)]
pub struct Tag {
    pub id: i64,
    pub fav_id: i64,
    pub label: String,
}

impl Tag {
    pub async fn create(client: &mut Client, fav_id: i64, label: &str) -> Result<Self, DbError> {
        Ok(Self::from_row(
            client
                .query_one(
                    "INSERT INTO tags (fav_id, label) VALUES ($1, $2) RETURNING *",
                    &[&fav_id, &label],
                )
                .await
                .map_err(|e| e.to_string())?,
        )?)
    }

    pub async fn delete(client: &mut Client, fav_id: i64) -> Result<u64, DbError> {
        Ok(client
            .execute("DELETE FROM tags WHERE fav_id = $1", &[&fav_id])
            .await
            .map_err(|e| e.to_string())?)
    }

    pub async fn of_user(client: &mut Client, user_id: i64) -> Result<Vec<Self>, DbError> {
        Ok(client.query("SELECT tags.id, tags.fav_id, tags.label FROM tags INNER JOIN favs ON tags.fav_id = favs.id WHERE favs.user_id = $1",
            &[&user_id]).await.map_err(|e| e.to_string())?.into_iter().map(Self::from_row).collect::<Result<Vec<_>, DbError>>()?)
    }

    fn from_row(row: Row) -> Result<Self, DbError> {
        Ok(Self {
            id: row.try_get("id").map_err(|e| e.to_string())?,
            fav_id: row.try_get("fav_id").map_err(|e| e.to_string())?,
            label: row.try_get("label").map_err(|e| e.to_string())?,
        })
    }
}
