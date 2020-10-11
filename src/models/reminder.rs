use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use sqlx::Done;

pub type DbError = sqlx::Error;

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Reminder {
    pub id: i64,
    pub channel_id: i64,
    pub source_msg_id: i64,
    pub user_id: i64,
    pub end_time: DateTime<Utc>,
    pub msg: String,
}

impl Reminder {
    pub async fn get(pool: &PgPool, source_msg_id: i64) -> Result<Self, DbError> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM reminders WHERE source_msg_id = $1",
            source_msg_id
        )
        .fetch_one(pool)
        .await
    }

    pub async fn list(pool: &PgPool) -> Result<Vec<Self>, DbError> {
        sqlx::query_as!(Self, "SELECT * FROM reminders")
            .fetch_all(pool)
            .await
    }

    pub async fn create(
        pool: &PgPool,
        channel_id: i64,
        source_msg_id: i64,
        user_id: i64,
        end_time: DateTime<Utc>,
        msg: &str,
    ) -> Result<Self, DbError> {
        sqlx::query_as!(
            Self,
            "INSERT INTO reminders (channel_id, source_msg_id, user_id, end_time, msg) VALUES ($1,$2,$3,$4,$5) RETURNING *",
            channel_id,
            source_msg_id,
            user_id,
            end_time, msg
        )
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, source_msg_id: i64) -> Result<u64, DbError> {
        Ok(sqlx::query!(
            "DELETE FROM reminders WHERE source_msg_id = $1",
            source_msg_id
        )
        .execute(pool)
        .await?
        .rows_affected())
    }
}
