use sqlx::postgres::PgPool;

pub type DbError = sqlx::Error;

pub async fn run(conn: &PgPool) -> Result<(), DbError> {
    //TODO: embed and run migrations
    Ok(())
}
