use futures::stream::TryStreamExt;
use sqlx::postgres::PgPool;

pub type DbError = sqlx::Error;

#[derive(sqlx::FromRow, Debug)]
pub struct Tag {
    pub id: i64,
    pub fav_id: i64,
    pub label: String,
}

impl Tag {
    pub async fn create(pool: &PgPool, fav_id: i64, label: &str) -> Result<Self, DbError> {
        sqlx::query_as!(
            Self,
            "INSERT INTO tags (fav_id, label) VALUES ($1,$2) RETURNING *",
            fav_id,
            label,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, fav_id: i64) -> Result<u64, DbError> {
        Ok(sqlx::query!("DELETE FROM tags WHERE fav_id = $1", fav_id)
            .execute(pool)
            .await?
            .rows_affected())
    }

    pub async fn of_user(pool: &PgPool, user_id: i64) -> Result<Vec<Self>, DbError> {
        Ok(sqlx::query_as!(Self, "SELECT tags.id, tags.fav_id, tags.label FROM tags INNER JOIN favs ON tags.fav_id = favs.id WHERE favs.user_id = $1", user_id)
            .fetch(pool).try_collect::<Vec<_>>().await?)
    }
}
