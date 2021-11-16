use chrono::{DateTime, Utc};
use sqlx::postgres::PgPool;
use twilight_model::id::{ChannelId, MessageId, UserId};

/// type alias for db driver specific error
pub type DbError = sqlx::Error;

/// Model struct for storage in db
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Reminder {
    /// unique identifier for a reminder
    pub id: i64,
    /// channel id
    pub channel_id: i64,
    /// user id
    pub user_id: i64,
    /// anchor id
    pub anchor_id: i64,
    /// when the reminder ends
    pub end_time: DateTime<Utc>,
    /// a message that should be sent to the user at the designated time
    pub msg: String,
}

impl Reminder {
    /// listing all reminders
    pub async fn list(pool: &PgPool) -> Result<Vec<Self>, DbError> {
        sqlx::query_as!(Self, "SELECT * FROM reminders")
            .fetch_all(pool)
            .await
    }

    /// creating a reminder
    pub async fn create(
        pool: &PgPool,
        channel_id: ChannelId,
        user_id: UserId,
        anchor_id: MessageId,
        end_time: DateTime<Utc>,
        msg: &str,
    ) -> Result<Self, DbError> {
        sqlx::query_as!(
            Self,
            "INSERT INTO reminders (channel_id, user_id, anchor_id, end_time, msg) VALUES ($1,$2,$3,$4,$5) RETURNING *",
            channel_id.0.get() as i64,
            user_id.0.get() as i64,
            anchor_id.0.get() as i64,
            end_time,
            msg
        )
        .fetch_one(pool)
        .await
    }

    /// deleting a reminder
    pub async fn delete(pool: &PgPool, id: i64) -> Result<u64, DbError> {
        Ok(sqlx::query!("DELETE FROM reminders WHERE id = $1", id,)
            .execute(pool)
            .await?
            .rows_affected())
    }
}
