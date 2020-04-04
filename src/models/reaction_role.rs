use tokio_postgres::{row::Row, Client};

pub type DbError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug)]
pub struct ReactionRole {
    pub id: i64,
    pub server_id: i64,
    pub role_id: i64,
    pub role_name: String,
    pub role_group: String,
    pub emoji: String,
}

impl ReactionRole {
    pub async fn get(
        client: &mut Client,
        server_id: i64,
        role_name: String,
    ) -> Result<Self, DbError> {
        Ok(Self::from_row(
            client
                .query_one(
                    "SELECT * FROM reaction_roles WHERE server_id = $1 AND role_name = $2",
                    &[&server_id, &role_name],
                )
                .await?,
        )?)
    }

    pub async fn list(client: &mut Client) -> Result<Vec<Self>, DbError> {
        Ok(client
            .query("SELECT * FROM reaction_roles", &[])
            .await?
            .into_iter()
            .map(Self::from_row)
            .collect::<Result<Vec<_>, DbError>>()?)
    }

    pub async fn list_by_emoji(client: &mut Client, emoji: &str) -> Result<Vec<Self>, DbError> {
        Ok(client
            .query("SELECT * FROM reaction_roles WHERE emoji = $1", &[&emoji])
            .await?
            .into_iter()
            .map(Self::from_row)
            .collect::<Result<Vec<_>, DbError>>()?)
    }

    pub async fn create(
        client: &mut Client,
        server_id: i64,
        role_id: i64,
        role_name: String,
        role_group: String,
        emoji: String,
    ) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "INSERT INTO reaction_roles (server_id, role_id, role_name, role_group, emoji) VALUES ($1, $2, $3, $4, $5) RETURNING *",
            &[&server_id, &role_id, &role_name, &role_group, &emoji],
        ).await?)?)
    }

    pub async fn delete(client: &mut Client, server_id: i64, role_id: i64) -> Result<u64, DbError> {
        Ok(client
            .execute(
                "DELETE FROM reaction_roles WHERE server_id = $1 AND role_id = $2",
                &[&server_id, &role_id],
            )
            .await?)
    }

    fn from_row(row: Row) -> Result<Self, DbError> {
        Ok(Self {
            id: row.try_get("id").map_err(|e| e.to_string())?,
            server_id: row.try_get("server_id").map_err(|e| e.to_string())?,
            role_id: row.try_get("role_id").map_err(|e| e.to_string())?,
            role_name: row.try_get("role_name").map_err(|e| e.to_string())?,
            role_group: row.try_get("role_group").map_err(|e| e.to_string())?,
            emoji: row.try_get("emoji").map_err(|e| e.to_string())?,
        })
    }
}
