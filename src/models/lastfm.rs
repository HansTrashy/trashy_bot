use tokio_postgres::{row::Row, Client};

pub type DbError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub struct Lastfm {
    pub id: i64,
    pub server_id: i64, //TODO remove this unnecessary field
    pub user_id: i64,
    pub username: String,
}

impl Lastfm {
    pub async fn get(client: &mut Client, user_id: i64) -> Result<Self, DbError> {
        Ok(Self::from_row(
            client
                .query_one("SELECT * FROM lastfms WHERE user_id = $1", &[&user_id])
                .await?,
        )?)
    }

    pub async fn create(
        client: &mut Client,
        user_id: i64,
        username: String,
    ) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "INSERT INTO lastfms (server_id, user_id, username) VALUES (0, $1, $2) RETURNING *",
            &[&user_id, &username],
        ).await?)?)
    }

    pub async fn update(
        client: &mut Client,
        user_id: i64,
        username: String,
    ) -> Result<Self, DbError> {
        Ok(Self::from_row(
            client
                .query_one(
                    "UPDATE lastfms SET username = $2 WHERE user_id = $1",
                    &[&user_id, &username],
                )
                .await?,
        )?)
    }

    // pub async fn delete(client: &mut Client, user_id: i64) -> Result<u64, DbError> {
    //     Ok(client
    //         .execute("DELETE FROM lastfms WHERE server_id = $1", &[&user_id])
    //         .await
    //         .map_err(|e| e.to_string())?)
    // }

    fn from_row(row: Row) -> Result<Self, DbError> {
        Ok(Self {
            id: row.try_get("id").map_err(|e| e.to_string())?,
            server_id: row.try_get("server_id").map_err(|e| e.to_string())?,
            user_id: row.try_get("user_id").map_err(|e| e.to_string())?,
            username: row.try_get("username").map_err(|e| e.to_string())?,
        })
    }
}
