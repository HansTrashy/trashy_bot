use chrono::NaiveDateTime;
use tokio_postgres::{row::Row, Client};

pub type DbError = String;

#[derive(Debug)]
pub struct Bank {
    pub id: i64,
    pub user_id: i64,
    pub user_name: String,
    pub amount: i64,
    pub last_payday: NaiveDateTime,
}

impl Bank {
    pub async fn get(client: &mut Client, user_id: i64) -> Result<Self, DbError> {
        Ok(Self::from_row(
            client
                .query_one("SELECT * FROM banks WHERE user_id = $1", &[&user_id])
                .await
                .map_err(|e| e.to_string())?,
        )?)
    }

    pub async fn top10(client: &mut Client) -> Result<Vec<Self>, DbError> {
        Ok(client
            .query("SELECT * FROM banks ORDER BY amount DESC LIMIT 10", &[])
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(Self::from_row)
            .collect::<Result<Vec<_>, DbError>>()?)
    }

    pub async fn create(
        client: &mut Client,
        user_id: i64,
        user_name: String,
        amount: i64,
        last_payday: NaiveDateTime,
    ) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "INSERT INTO banks (user_id, user_name, amount, last_payday) VALUES ($1, $2, $3, $4) RETURNING *",
            &[&user_id, &user_name, &amount, &last_payday],
        ).await.map_err(|e| e.to_string())?)?)
    }

    pub async fn update(
        client: &mut Client,
        user_id: i64,
        amount: i64,
        last_payday: NaiveDateTime,
    ) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "UPDATE banks SET (amount, last_payday) = ($2, $3) WHERE user_id = $1 RETURNING *",
            &[&user_id, &amount, &last_payday],
        ).await.map_err(|e| e.to_string())?)?)
    }

    fn from_row(row: Row) -> Result<Self, DbError> {
        Ok(Self {
            id: row.try_get("id").map_err(|e| e.to_string())?,
            user_id: row.try_get("user_id").map_err(|e| e.to_string())?,
            user_name: row.try_get("user_name").map_err(|e| e.to_string())?,
            amount: row.try_get("amount").map_err(|e| e.to_string())?,
            last_payday: row.try_get("last_payday").map_err(|e| e.to_string())?,
        })
    }
}
