use chrono::NaiveDateTime;
use sqlx::postgres::PgPool;

pub type DbError = sqlx::Error;

#[derive(Debug, sqlx::FromRow)]
pub struct Bank {
    pub id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub amount: i64,
    pub last_payday: NaiveDateTime,
}

impl Bank {
    pub async fn get(pool: &PgPool, user_id: i64) -> Result<Self, DbError> {
        sqlx::query_as!(Self, "SELECT * FROM banks WHERE user_id = $1", user_id)
            .fetch_one(pool)
            .await
    }

    pub async fn top10(pool: &PgPool) -> Result<Vec<Self>, DbError> {
        sqlx::query_as!(Self, "SELECT * FROM banks ORDER BY amount DESC LIMIT 10")
            .fetch_all(pool)
            .await
    }

    pub async fn create(
        pool: &PgPool,
        user_id: i64,
        user_name: String,
        amount: i64,
        last_payday: NaiveDateTime,
    ) -> Result<Self, DbError> {
        sqlx::query_as!(
            Self,
            "INSERT INTO banks (user_id, user_name, amount, last_payday) VALUES ($1,$2,$3,$4) RETURNING *",
            user_id,
            user_name,
            amount,
            last_payday
        )
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &PgPool,
        user_id: i64,
        amount: i64,
        last_payday: NaiveDateTime,
    ) -> Result<Self, DbError> {
        sqlx::query_as!(
            Self,
            "UPDATE banks SET (amount, last_payday) = ($1,$2) WHERE user_id = $3 RETURNING *",
            amount,
            last_payday,
            user_id,
        )
        .fetch_one(pool)
        .await
    }
}
