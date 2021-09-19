//! Trashy Bot

use anyhow::{Context, Result};
use trashy_bot::{config::Config, TrashyBot};

#[tokio::main]
async fn main() -> Result<()> {
    let config: Config = toml::from_str(
        &tokio::fs::read_to_string("config.toml")
            .await
            .context("Could not load config file")?,
    )
    .context("Failed to parse config")?;

    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    TrashyBot::run(config).await.context("failed to run bot")?;

    Ok(())
}
