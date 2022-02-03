use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;

pub type DbError = sqlx::Error;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Mute {
    pub id: i64,
    pub server_id: i64,
    pub user_id: i64,
    pub end_time: DateTime<Utc>,
}

impl Mute {
    pub async fn get(pool: &PgPool, server_id: i64, user_id: i64) -> Result<Self, DbError> {
        sqlx::query_as::<_, Self>("SELECT * FROM mutes WHERE user_id = $1 AND server_id = $2")
            .bind(user_id)
            .bind(server_id)
            .fetch_one(pool)
            .await
    }

    pub async fn list(pool: &PgPool) -> Result<Vec<Self>, DbError> {
        sqlx::query_as::<_, Self>("SELECT * FROM mutes")
            .fetch_all(pool)
            .await
    }

    pub async fn create(
        pool: &PgPool,
        server_id: i64,
        user_id: i64,
        end_time: DateTime<Utc>,
    ) -> Result<Self, DbError> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO mutes (server_id, user_id, end_time) VALUES ($1,$2,$3) RETURNING *",
        )
        .bind(server_id)
        .bind(user_id)
        .bind(end_time)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, server_id: i64, user_id: i64) -> Result<u64, DbError> {
        Ok(
            sqlx::query("DELETE FROM mutes WHERE server_id = $1 AND user_id = $2")
                .bind(server_id)
                .bind(user_id)
                .execute(pool)
                .await?
                .rows_affected(),
        )
    }
}
