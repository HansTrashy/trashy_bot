use chrono::{DateTime, Utc};
use tokio_postgres::{row::Row, Client};

pub type DbError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone, Debug)]
pub struct Reminder {
    pub id: i64,
    pub channel_id: i64,
    pub source_msg_id: i64,
    pub user_id: i64,
    pub end_time: DateTime<Utc>,
    pub msg: String,
}

impl Reminder {
    pub async fn get(client: &mut Client, source_msg_id: i64) -> Result<Self, DbError> {
        Ok(Self::from_row(
            client
                .query_one(
                    "SELECT * FROM reminders WHERE source_msg_id = $1",
                    &[&source_msg_id],
                )
                .await?,
        )?)
    }

    pub async fn list(client: &mut Client) -> Result<Vec<Self>, DbError> {
        Ok(client
            .query("SELECT * FROM reminders", &[])
            .await?
            .into_iter()
            .map(Self::from_row)
            .collect::<Result<Vec<_>, DbError>>()?)
    }

    pub async fn create(
        client: &mut Client,
        channel_id: i64,
        source_msg_id: i64,
        user_id: i64,
        end_time: DateTime<Utc>,
        msg: &str,
    ) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "INSERT INTO reminders (channel_id, source_msg_id, user_id, end_time, msg) VALUES ($1, $2, $3, $4, $5) RETURNING *",
            &[&channel_id, &source_msg_id, &user_id, &end_time, &msg],
        ).await?)?)
    }

    pub async fn delete(client: &mut Client, source_msg_id: i64) -> Result<u64, DbError> {
        Ok(client
            .execute(
                "DELETE FROM reminders WHERE source_msg_id = $1",
                &[&source_msg_id],
            )
            .await?)
    }

    fn from_row(row: Row) -> Result<Self, DbError> {
        Ok(Self {
            id: row.try_get("id").map_err(|e| e.to_string())?,
            channel_id: row.try_get("channel_id").map_err(|e| e.to_string())?,
            source_msg_id: row.try_get("source_msg_id").map_err(|e| e.to_string())?,
            user_id: row.try_get("user_id").map_err(|e| e.to_string())?,
            end_time: row.try_get("end_time").map_err(|e| e.to_string())?,
            msg: row.try_get("msg").map_err(|e| e.to_string())?,
        })
    }
}
