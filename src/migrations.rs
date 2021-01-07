use sqlx::postgres::PgPool;

pub type DbError = sqlx::Error;

#[allow(unused)]
pub async fn run(conn: &PgPool) -> Result<(), DbError> {
    //TODO: embed and run migrations
    Ok(())
}
