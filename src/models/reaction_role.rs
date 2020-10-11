use sqlx::postgres::PgPool;
use sqlx::Done;

pub type DbError = sqlx::Error;

#[derive(Debug, sqlx::FromRow)]
pub struct ReactionRole {
    pub id: i64,
    pub server_id: i64,
    pub role_id: i64,
    pub role_name: String,
    pub role_group: String,
    pub emoji: String,
    pub role_description: Option<String>,
}

impl ReactionRole {
    pub async fn get(pool: &PgPool, server_id: i64, role_name: String) -> Result<Self, DbError> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM reaction_roles WHERE server_id = $1 AND role_name = $2",
            server_id,
            role_name,
        )
        .fetch_one(pool)
        .await
    }

    pub async fn list(pool: &PgPool) -> Result<Vec<Self>, DbError> {
        sqlx::query_as!(Self, "SELECT * FROM reaction_roles")
            .fetch_all(pool)
            .await
    }

    pub async fn list_by_emoji(pool: &PgPool, emoji: &str) -> Result<Vec<Self>, DbError> {
        sqlx::query_as!(Self, "SELECT * FROM reaction_roles WHERE emoji = $1", emoji)
            .fetch_all(pool)
            .await
    }

    pub async fn create(
        pool: &PgPool,
        server_id: i64,
        role_id: i64,
        role_name: String,
        role_group: String,
        emoji: String,
        description: Option<String>,
    ) -> Result<Self, DbError> {
        sqlx::query_as!(
            Self,
            "INSERT INTO reaction_roles (server_id, role_id, role_name, role_group, emoji, role_description) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
            server_id,
            role_id,
            role_name,
            role_group,
            emoji,
            description,
        )
        .fetch_one(pool).await
    }

    pub async fn change_description(
        pool: &PgPool,
        server_id: i64,
        role_id: i64,
        description: Option<String>,
    ) -> Result<u64, DbError> {
        Ok(sqlx::query!(
            "UPDATE reaction_roles SET role_description = $1 WHERE server_id = $2 AND role_id = $3",
            description,
            server_id,
            role_id,
        )
        .execute(pool)
        .await?
        .rows_affected())
    }

    pub async fn delete(pool: &PgPool, server_id: i64, role_id: i64) -> Result<u64, DbError> {
        Ok(sqlx::query!(
            "DELETE FROM reaction_roles WHERE server_id = $1 AND role_id = $2",
            server_id,
            role_id
        )
        .execute(pool)
        .await?
        .rows_affected())
    }
}
