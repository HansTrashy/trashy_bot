use sqlx::postgres::PgPool;

pub type DbError = sqlx::Error;

#[derive(Debug, sqlx::FromRow)]
pub struct Shiny {
    pub id: i64,
    pub server_id: i64,
    pub user_id: i64,
    pub username: String,
    pub amount: i64,
}

impl Shiny {
    pub async fn get(pool: &PgPool, user_id: i64) -> Result<Self, DbError> {
        sqlx::query_as::<_, Self>("SELECT * FROM shinys WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await
    }

    pub async fn list(pool: &PgPool, server_id: i64) -> Result<Vec<Self>, DbError> {
        sqlx::query_as::<_, Self>("SELECT * FROM shinys WHERE server_id = $1")
            .bind(server_id)
            .fetch_all(pool)
            .await
    }

    pub async fn create(
        pool: &PgPool,
        server_id: i64,
        user_id: i64,
        username: String,
        amount: i64,
    ) -> Result<Self, DbError> {
        sqlx::query_as::<_, Self>("INSERT INTO shinys (server_id, user_id, username, amount) VALUES ($1, $2, $3, $4) RETURNING *")
            .bind(server_id)
            .bind(user_id)
            .bind(username)
            .bind(amount)
            .fetch_one(pool).await
    }

    pub async fn update(pool: &PgPool, user_id: i64, amount: i64) -> Result<Self, DbError> {
        sqlx::query_as::<_, Self>("UPDATE shinys SET amount = $1 WHERE user_id = $2 RETURNING *")
            .bind(amount)
            .bind(user_id)
            .fetch_one(pool)
            .await
    }

    pub async fn delete(pool: &PgPool, user_id: i64) -> Result<u64, DbError> {
        Ok(sqlx::query("DELETE FROM shinys WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await?
            .rows_affected())
    }
}
