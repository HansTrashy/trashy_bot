use tokio_postgres::{row::Row, Client};

pub type DbError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Clone)]
pub struct Fav {
    pub id: i64,
    pub server_id: i64,
    pub channel_id: i64,
    pub msg_id: i64,
    pub user_id: i64,
    pub author_id: i64,
}

impl Fav {
    pub async fn list(client: &mut Client, user_id: i64) -> Result<Vec<Self>, DbError> {
        Ok(client
            .query("SELECT * FROM favs WHERE user_id = $1", &[&user_id])
            .await?
            .into_iter()
            .map(Self::from_row)
            .collect::<Result<Vec<_>, DbError>>()?)
    }

    pub async fn create(
        client: &mut Client,
        server_id: i64,
        channel_id: i64,
        msg_id: i64,
        user_id: i64,
        author_id: i64,
    ) -> Result<Self, DbError> {
        Ok(Self::from_row(client.query_one(
            "INSERT INTO favs (server_id, channel_id, msg_id, user_id, author_id) VALUES ($1, $2, $3, $4, $5) RETURNING *",
            &[&server_id, &channel_id, &msg_id, &user_id, &author_id],
        ).await?)?)
    }

    pub async fn delete(client: &mut Client, id: i64) -> Result<u64, DbError> {
        Ok(client
            .execute("DELETE FROM favs WHERE id = $1", &[&id])
            .await?)
    }

    pub async fn untagged(client: &mut Client, user_id: i64) -> Result<Vec<Self>, DbError> {
        Ok(client.query("SELECT favs.id, favs.server_id, favs.channel_id, favs.msg_id, favs.user_id, favs.author_id FROM favs LEFT JOIN tags ON favs.id = tags.fav_id WHERE favs.user_id = $1 AND tags.id IS NULL",
            &[&user_id]).await?.into_iter().map(Self::from_row).collect::<Result<Vec<_>, DbError>>()?)
    }

    pub async fn tagged_with(
        client: &mut Client,
        user_id: i64,
        tags: Vec<String>,
    ) -> Result<Vec<Self>, DbError> {
        Ok(client.query("SELECT favs.id, favs.server_id, favs.channel_id, favs.msg_id, favs.user_id, favs.author_id FROM favs INNER JOIN tags ON favs.id = tags.fav_id WHERE favs.user_id = $1 AND tags.label = ANY($2)",
            &[&user_id, &tags]).await?.into_iter().map(Self::from_row).collect::<Result<Vec<_>, DbError>>()?)
    }

    fn from_row(row: Row) -> Result<Self, DbError> {
        Ok(Self {
            id: row.try_get("id").map_err(|e| e.to_string())?,
            server_id: row.try_get("server_id").map_err(|e| e.to_string())?,
            channel_id: row.try_get("channel_id").map_err(|e| e.to_string())?,
            msg_id: row.try_get("msg_id").map_err(|e| e.to_string())?,
            user_id: row.try_get("user_id").map_err(|e| e.to_string())?,
            author_id: row.try_get("author_id").map_err(|e| e.to_string())?,
        })
    }
}
