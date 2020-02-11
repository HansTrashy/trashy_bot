use postgres::Client;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!();
}

pub type DbError = Box<dyn std::error::Error>;

pub fn run(conn: &mut Client) -> Result<(), DbError> {
    embedded::migrations::runner().run(&mut *conn)?;
    Ok(())
}
