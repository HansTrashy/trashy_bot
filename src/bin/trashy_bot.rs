//! Trashy Bot

use trashy_bot::{config::Config, TrashyBot};

#[tokio::main]
async fn main() {
    let config: Config = toml::from_str(
        &tokio::fs::read_to_string("config.toml")
            .await
            .expect("Could not load config file"),
    )
    .expect("Failed to parse config");

    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    TrashyBot::run(config).await.expect("failed to run bot");
}
