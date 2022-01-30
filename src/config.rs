//! TrashyBot Configuration

use serde::Deserialize;

#[derive(Debug, Deserialize)]
/// all necessary configuration parameters for the bot
pub struct Config {
    /// a log level setting, may be a simple `info`, `debug`, `error` or more specific like `info,trashy_bot=debug`
    pub log_level: String,
    // pub prefix: String,
    // pub delimiter: char,
    /// the discord bot token
    pub discord_token: String,
    // pub lastfm_api_key: String,
    // pub xkcd_index: String,
    /// database url
    pub db_url: String,
    /// maximum pool size for the database
    pub db_pool_max_size: u32,
}
