use tokio_postgres::{row::Row, Client};

pub type DbError = String;

#[derive(Debug)]
pub struct Shiny {
    pub id: i64,
    pub server_id: i64,
    pub user_id: i64,
    pub username: String,
    pub amount: i64,
}

impl Shiny {
    pub async fn get(client: &mut Client, user_id: i64) -> Result<Self, DbError> {
        Ok(Self::from_row(
            client
                .query_one("SELECT * FROM shinys WHERE user_id = $1", &[&user_id])
                .await
                .map_err(|e| e.to_string())?,
        )?)
    }

    pub async fn list(client: &mut Client, server_id: i64) -> Result<Vec<Self>, DbError> {
        Ok(client
            .query("SELECT * FROM shinys WHERE server_id = $1", &[&server_id])
            .await
            .map_err(|e| e.to_string())?
            .into_iter()
            .map(Self::from_row)
            .collect::<Result<Vec<_>, DbError>>()?)
    }

    pub async fn create(
        client: &mut Client,
        server_id: i64,
        user_id: i64,
        username: String,
        amount: i64,
    ) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "INSERT INTO shinys (server_id, user_id, username, amount) VALUES ($1, $2, $3, $4) RETURNING *",
            &[&server_id, &user_id, &username, &amount],
        ).await.map_err(|e| e.to_string())?)?)
    }

    pub async fn update(client: &mut Client, user_id: i64, amount: i64) -> Result<Self, DbError> {
        Ok(Self::from_row(
            client
                .query_one(
                    "UPDATE shinys SET amount = $2 WHERE user_id = $1",
                    &[&user_id, &amount],
                )
                .await
                .map_err(|e| e.to_string())?,
        )?)
    }

    pub async fn delete(client: &mut Client, user_id: i64) -> Result<u64, DbError> {
        Ok(client
            .execute("DELETE FROM shinys WHERE user_id = $2", &[&user_id])
            .await
            .map_err(|e| e.to_string())?)
    }

    fn from_row(row: Row) -> Result<Self, DbError> {
        Ok(Self {
            id: row.try_get("id").map_err(|e| e.to_string())?,
            server_id: row.try_get("server_id").map_err(|e| e.to_string())?,
            user_id: row.try_get("user_id").map_err(|e| e.to_string())?,
            username: row.try_get("username").map_err(|e| e.to_string())?,
            amount: row.try_get("amount").map_err(|e| e.to_string())?,
        })
    }
}
