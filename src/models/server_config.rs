use sqlx::postgres::PgPool;

pub type DbError = sqlx::Error;

#[derive(Debug, sqlx::FromRow)]
pub struct ServerConfig {
    pub id: i64,
    pub server_id: i64,
    pub config: serde_json::Value,
}

impl ServerConfig {
    pub async fn get(pool: &PgPool, server_id: i64) -> Result<Self, DbError> {
        sqlx::query_as::<_, Self>("SELECT * FROM server_configs WHERE server_id = $1")
            .bind(server_id)
            .fetch_one(pool)
            .await
    }

    pub async fn list(pool: &PgPool) -> Result<Vec<Self>, DbError> {
        sqlx::query_as::<_, Self>("SELECT * FROM server_configs")
            .fetch_all(pool)
            .await
    }

    pub async fn create(
        pool: &PgPool,
        server_id: i64,
        config: serde_json::Value,
    ) -> Result<Self, DbError> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO server_configs (server_id, config) VALUES ($1,$2) RETURNING *",
        )
        .bind(server_id)
        .bind(config)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        server_id: i64,
        config: serde_json::Value,
    ) -> Result<Self, DbError> {
        sqlx::query_as::<_, Self>(
            "UPDATE server_configs SET config = $1 WHERE server_id = $2 RETURNING *",
        )
        .bind(config)
        .bind(server_id)
        .fetch_one(pool)
        .await
    }
}
