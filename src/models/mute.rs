use chrono::{DateTime, Utc};
use tokio_postgres::{row::Row, Client};

pub type DbError = String;

#[derive(Debug)]
pub struct Mute {
    pub id: i64,
    pub server_id: i64,
    pub user_id: i64,
    pub end_time: DateTime<Utc>,
}

impl Mute {
    pub async fn get(client: &mut Client, server_id: i64, user_id: i64) -> Result<Self, DbError> {
        Ok(Self::from_row(
            client
                .query_one(
                    "SELECT * FROM mutes WHERE user_id = $1 AND server_id = $2",
                    &[&user_id, &server_id],
                )
                .await
                .map_err(|e| e.to_string())?,
        )?)
    }

    pub async fn create(
        client: &mut Client,
        server_id: i64,
        user_id: i64,
        end_time: DateTime<Utc>,
    ) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "INSERT INTO mutes (server_id, user_id, end_time) VALUES ($1, $2, $3) RETURNING *",
            &[&server_id, &user_id, &end_time],
        ).await.map_err(|e| e.to_string())?)?)
    }

    // pub async fn update(
    //     client: &mut Client,
    //     server_id: i64,
    //     user_id: i64,
    //     end_time: DateTime<Utc>,
    // ) -> Result<Self, DbError> {
    //     Ok(Self::from_row(
    //         client
    //             .query_one(
    //                 "UPDATE mutes SET end_time = $3 WHERE server_id = $1 AND user_id = $1",
    //                 &[&server_id, &user_id, &end_time],
    //             )
    //             .await
    //             .map_err(|e| e.to_string())?,
    //     )?)
    // }

    pub async fn delete(client: &mut Client, server_id: i64, user_id: i64) -> Result<u64, DbError> {
        Ok(client
            .execute(
                "DELETE FROM mutes WHERE server_id = $1 AND user_id = $2",
                &[&server_id, &user_id],
            )
            .await
            .map_err(|e| e.to_string())?)
    }

    fn from_row(row: Row) -> Result<Self, DbError> {
        Ok(Self {
            id: row.try_get("id").map_err(|e| e.to_string())?,
            server_id: row.try_get("server_id").map_err(|e| e.to_string())?,
            user_id: row.try_get("user_id").map_err(|e| e.to_string())?,
            end_time: row.try_get("end_time").map_err(|e| e.to_string())?,
        })
    }
}
