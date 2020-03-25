use tokio_postgres::Client;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!();
}

pub type DbError = Box<dyn std::error::Error>;

pub async fn run(conn: &mut Client) -> Result<(), DbError> {
    embedded::migrations::runner().run_async(&mut *conn).await?;
    Ok(())
}
