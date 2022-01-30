//! TrashyBot error types

use thiserror::Error;
use twilight_gateway::cluster::ClusterStartError;
use twilight_http::response::DeserializeBodyError;
use twilight_http::Error as TwilightError;
use twilight_validate::message::MessageValidationError;

#[derive(Error, Debug)]
/// this error enum contains all variants how the bot startup may fail
pub enum TrashyStartupError {
    /// the twilight cluster could not be started
    #[error("Cluster startup failed: {0}")]
    ClusterStartupFailed(#[from] ClusterStartError),
    /// generic twilight error
    #[error("Twilight Error: {0}")]
    TwilightError(#[from] TwilightError),
    /// deserialization failed
    #[error("Deserialization failure: {0}")]
    DeserializeFailed(#[from] DeserializeBodyError),
    #[error("Database Pool failed: {0}")]
    /// database connection failed
    DatabaseConnectionFailed(#[from] sqlx::Error),
    #[error("Database Migration failed: {0}")]
    /// database migration failed
    DatabaseMigrationFailed(#[from] sqlx::migrate::MigrateError),
}

#[derive(Error, Debug)]
/// this error enum contains all possible ways in which command executions may fail
pub enum TrashyCommandError {
    /// unknown command that the bot does not support
    #[error("The `{0}` command is unknown")]
    UnknownCommand(String),
    /// unknown option that the command does not support
    #[error("The `{0}` command option is unknown")]
    UnknownOption(String),
    /// missing option that the command needs
    #[error("The command option is needed")]
    MissingOption,
    /// database error
    #[error("Database failure: {0}")]
    Database(#[from] sqlx::Error),
    /// missing data
    #[error("Missing data: {0}")]
    MissingData(String),
    /// deserialize model error
    #[error("model could not be deserialized: {0}")]
    DeserializeModel(#[from] DeserializeBodyError),
    /// message validation error
    #[error("validation error: {0}")]
    Validation(#[from] MessageValidationError),
    /// http error
    #[error("communication with discord failed: {0}")]
    Http(#[from] twilight_http::Error),
}
