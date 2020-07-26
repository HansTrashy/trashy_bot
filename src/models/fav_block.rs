use tokio_postgres::{row::Row, Client};

pub type DbError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone)]
pub struct FavBlock {
    pub id: i64,
    pub server_id: i64,  // server on which this fav is blocked
    pub channel_id: i64, // blocked channel id
    pub msg_id: i64,     // blocked message id
}

impl FavBlock {
    pub async fn list(client: &mut Client, server_id: i64) -> Result<Vec<Self>, DbError> {
        Ok(client
            .query(
                "SELECT * FROM fav_blocks WHERE server_id = $1",
                &[&server_id],
            )
            .await?
            .into_iter()
            .map(Self::from_row)
            .collect::<Result<Vec<_>, DbError>>()?)
    }

    pub async fn create(
        client: &mut Client,
        server_id: i64,
        channel_id: i64,
        msg_id: i64,
    ) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "INSERT INTO fav_blocks (server_id, channel_id, msg_id) VALUES ($1, $2, $3) RETURNING *",
            &[&server_id, &channel_id, &msg_id],
        ).await?)?)
    }

    pub async fn delete(client: &mut Client, id: i64) -> Result<u64, DbError> {
        Ok(client
            .execute("DELETE FROM fav_blocks WHERE id = $1", &[&id])
            .await?)
    }

    fn from_row(row: Row) -> Result<Self, DbError> {
        Ok(Self {
            id: row.try_get("id").map_err(|e| e.to_string())?,
            server_id: row.try_get("server_id").map_err(|e| e.to_string())?,
            channel_id: row.try_get("channel_id").map_err(|e| e.to_string())?,
            msg_id: row.try_get("msg_id").map_err(|e| e.to_string())?,
        })
    }
}
