use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub log_level: String,
    pub prefix: String,
    pub delimiter: char,
    pub discord_token: String,
    pub application_id: u64,
    pub lastfm_api_key: String,
    pub xkcd_index: String,
    pub db_url: String,
    pub db_pool_max_size: u32,
    pub buckets: Vec<Bucket>,
}

#[derive(Debug, Deserialize)]
pub struct Bucket {
    pub name: String,
    pub delay: Option<u64>,
    pub time_span: Option<u64>,
    pub limit: Option<u32>,
}
