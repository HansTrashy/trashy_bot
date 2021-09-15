use thiserror::Error;
use twilight_gateway::cluster::ClusterStartError;
use twilight_http::request::application::InteractionError;
use twilight_http::response::DeserializeBodyError;
use twilight_http::Error as TwilightError;

#[derive(Error, Debug)]
pub enum TrashyStartupError {
    #[error("Cluster startup failed: {0}")]
    ClusterStartupFailed(#[from] ClusterStartError),
    #[error("Interaction failed: {0}")]
    InteractionFailed(#[from] InteractionError),
    #[error("Twilight Error: {0}")]
    TwilightError(#[from] TwilightError),
    #[error("Deserialization failure: {0}")]
    DeserializeFailed(#[from] DeserializeBodyError),
}

#[derive(Error, Debug)]
pub enum TrashyCommandError {
    #[error("The `{0}` command is unknown")]
    UnknownCommand(String),
}
