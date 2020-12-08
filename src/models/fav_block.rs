use sqlx::postgres::PgPool;

pub type DbError = sqlx::Error;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FavBlock {
    pub id: i64,
    pub server_id: i64,  // server on which this fav is blocked
    pub channel_id: i64, // blocked channel id
    pub msg_id: i64,     // blocked message id
}

impl FavBlock {
    pub async fn check_blocked(pool: &PgPool, channel_id: i64, msg_id: i64) -> bool {
        match sqlx::query!(
            "SELECT * FROM fav_blocks WHERE channel_id = $1 AND msg_id = $2",
            channel_id,
            msg_id
        )
        .fetch_all(pool)
        .await
        {
            Ok(rows) => rows.len() > 0,
            Err(_) => false,
        }
    }

    pub async fn create(
        pool: &PgPool,
        server_id: i64,
        channel_id: i64,
        msg_id: i64,
    ) -> Result<Self, DbError> {
        sqlx::query_as!(
            Self,
            "INSERT INTO fav_blocks (server_id, channel_id, msg_id) VALUES ($1,$2,$3) RETURNING *",
            server_id,
            channel_id,
            msg_id,
        )
        .fetch_one(pool)
        .await
    }
}
