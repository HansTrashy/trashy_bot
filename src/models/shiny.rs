use postgres::{row::Row, Client};

pub type DbError = Box<dyn std::error::Error>;

#[derive(Debug)]
pub struct Shiny {
    pub id: i64,
    pub server_id: i64,
    pub user_id: i64,
    pub username: String,
    pub amount: i64,
}

impl Shiny {
    pub fn get(client: &mut Client, user_id: i64) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "SELECT * FROM shinys WHERE user_id = $1",
            &[&user_id],
        )?)?)
    }

    pub fn list(client: &mut Client, server_id: i64) -> Result<Vec<Self>, DbError> {
        Ok(client
            .query("SELECT * FROM shinys WHERE server_id = $1", &[&server_id])?
            .into_iter()
            .map(Self::from_row)
            .collect::<Result<Vec<_>, DbError>>()?)
    }

    pub fn create(
        client: &mut Client,
        server_id: i64,
        user_id: i64,
        username: String,
        amount: i64,
    ) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "INSERT INTO shinys VALUES (server_id, user_id, username, amount) = ($1, $2, $3, $4) RETURNING *",
            &[&server_id, &user_id, &username, &amount],
        )?)?)
    }

    pub fn update(client: &mut Client, user_id: i64, amount: i64) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "UPDATE shinys SET amount = $2 WHERE user_id = $1",
            &[&user_id, &amount],
        )?)?)
    }

    pub fn delete(client: &mut Client, user_id: i64) -> Result<u64, DbError> {
        Ok(client.execute("DELETE FROM shinys WHERE user_id = $2", &[&user_id])?)
    }

    fn from_row(row: Row) -> Result<Self, DbError> {
        Ok(Self {
            id: row.try_get("id")?,
            server_id: row.try_get("server_id")?,
            user_id: row.try_get("user_id")?,
            username: row.try_get("username")?,
            amount: row.try_get("amount")?,
        })
    }
}
