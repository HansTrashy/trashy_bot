use sqlx::postgres::PgPool;
use sqlx::Done;

pub type DbError = sqlx::Error;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Fav {
    pub id: i64,
    pub server_id: i64,
    pub channel_id: i64,
    pub msg_id: i64,
    pub user_id: i64,
    pub author_id: i64,
}

impl Fav {
    pub async fn list(
        pool: &PgPool,
        user_id: i64,
        server_id: Option<i64>,
    ) -> Result<Vec<Self>, DbError> {
        let server_id = server_id.unwrap_or(0);
        sqlx::query_as!(
            Self,
            "SELECT favs.id, favs.server_id, favs.channel_id, favs.msg_id, favs.user_id, favs.author_id FROM favs LEFT JOIN fav_blocks ON favs.server_id = fav_blocks.server_id AND favs.channel_id = fav_blocks.channel_id AND favs.msg_id = fav_blocks.msg_id WHERE user_id = $1 AND (fav_blocks.id IS NULL OR fav_blocks.server_id != $2)",
            user_id,
            server_id,
        )
        .fetch_all(pool)
        .await
    }

    pub async fn list_by_channel_msg(
        pool: &PgPool,
        channel_id: i64,
        msg_id: i64,
    ) -> Result<Vec<Self>, DbError> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM favs WHERE channel_id = $1 AND msg_id = $2",
            channel_id,
            msg_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn list_all_from_server(pool: &PgPool, server_id: i64) -> Result<Vec<Self>, DbError> {
        sqlx::query_as!(Self, "SELECT * FROM favs WHERE server_id = $1", server_id)
            .fetch_all(pool)
            .await
    }

    pub async fn create(
        pool: &PgPool,
        server_id: i64,
        channel_id: i64,
        msg_id: i64,
        user_id: i64,
        author_id: i64,
    ) -> Result<Self, DbError> {
        sqlx::query_as!(
            Self,
            "INSERT INTO favs (server_id, channel_id, msg_id, user_id, author_id) VALUES ($1, $2, $3, $4, $5) RETURNING *",
            server_id,
            channel_id,
            msg_id,
            user_id,
            author_id,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &PgPool, id: i64) -> Result<u64, DbError> {
        Ok(sqlx::query!("DELETE FROM favs WHERE id = $1", id)
            .execute(pool)
            .await?
            .rows_affected())
    }

    pub async fn untagged(pool: &PgPool, user_id: i64) -> Result<Vec<Self>, DbError> {
        sqlx::query_as!(
            Self,
            "SELECT favs.id, favs.server_id, favs.channel_id, favs.msg_id, favs.user_id, favs.author_id FROM favs LEFT JOIN tags ON favs.id = tags.fav_id WHERE favs.user_id = $1 AND tags.id IS NULL",
            user_id
        )
        .fetch_all(pool)
        .await
    }

    pub async fn tagged_with(
        pool: &PgPool,
        user_id: i64,
        server_id: Option<i64>,
        tags: Vec<String>,
    ) -> Result<Vec<Self>, DbError> {
        let server_id = server_id.unwrap_or(0);
        sqlx::query_as!(
            Self,
            "SELECT favs.id, favs.server_id, favs.channel_id, favs.msg_id, favs.user_id, favs.author_id FROM favs INNER JOIN tags ON favs.id = tags.fav_id LEFT JOIN fav_blocks ON favs.server_id = fav_blocks.server_id AND favs.channel_id = fav_blocks.channel_id AND favs.msg_id = fav_blocks.msg_id WHERE favs.user_id = $1 AND tags.label = ANY($2) AND (fav_blocks.id IS NULL OR fav_blocks.server_id != $3)",
            user_id,
            &tags,
            server_id,
        )
        .fetch_all(pool)
        .await
    }
}
