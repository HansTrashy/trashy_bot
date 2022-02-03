use sqlx::postgres::PgPool;

pub type DbError = sqlx::Error;

#[derive(Debug, sqlx::FromRow)]
pub struct Lastfm {
    pub id: i64,
    pub server_id: i64, //TODO remove this unnecessary field
    pub user_id: i64,
    pub username: String,
}

impl Lastfm {
    pub async fn get(pool: &PgPool, user_id: i64) -> Result<Self, DbError> {
        sqlx::query_as::<_, Self>("SELECT * FROM lastfms WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await
    }

    pub async fn create(pool: &PgPool, user_id: i64, username: String) -> Result<Self, DbError> {
        sqlx::query_as::<_, Self>(
            "INSERT INTO lastfms (server_id, user_id, username) VALUES (0, $1, $2) RETURNING *",
        )
        .bind(user_id)
        .bind(username)
        .fetch_one(pool)
        .await
    }

    pub async fn update(pool: &PgPool, user_id: i64, username: String) -> Result<Self, DbError> {
        sqlx::query_as::<_, Self>("UPDATE lastfms SET username = $1 WHERE user_id = $2 RETURNING *")
            .bind(username)
            .bind(user_id)
            .fetch_one(pool)
            .await
    }

    pub async fn delete(pool: &PgPool, user_id: i64) -> Result<u64, DbError> {
        Ok(sqlx::query("DELETE FROM lastfms WHERE user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await?
            .rows_affected())
    }
}
