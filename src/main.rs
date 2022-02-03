#![deny(clippy::all)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_wrap)]
#![deny(nonstandard_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(unused)]
// TODO: remove this when sqlx fixed the macro calls with `_expr`
#![allow(clippy::used_underscore_binding)]
//! Trashy Bot

#[macro_use]
extern crate tantivy;

mod commands;
mod config;
mod handler;
mod migrations;
mod models;
mod rules;
mod startup;
mod util;

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serenity::{
    client::bridge::gateway::GatewayIntents,
    client::bridge::gateway::ShardManager,
    client::Context,
    framework::standard::{
        help_commands,
        macros::{help, hook},
        Args, CommandGroup, CommandResult, DispatchError, HelpOptions, StandardFramework,
    },
    http::Http,
    model::{channel::Message, id::UserId},
    prelude::*,
};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, trace, warn};

struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct DatabasePool;
impl TypeMapKey for DatabasePool {
    type Value = PgPool;
}

struct Config;
impl TypeMapKey for Config {
    type Value = config::Config;
}

struct RulesState;
impl TypeMapKey for RulesState {
    type Value = Arc<Mutex<self::rules::State>>;
}

struct OptOut;
impl TypeMapKey for OptOut {
    type Value = Arc<Mutex<OptOutStore>>;
}

struct ReqwestClient;
impl TypeMapKey for ReqwestClient {
    type Value = reqwest::Client;
}

struct RunningState;
impl TypeMapKey for RunningState {
    type Value = BotState;
}

struct XkcdState;
impl TypeMapKey for XkcdState {
    type Value = XkcdIndexStorage;
}

struct BotState {
    running_since: std::time::Instant,
}

#[derive(Serialize, Deserialize)]
struct XkcdIndexStorage {
    pub indexed: u64,
}

impl XkcdIndexStorage {
    fn load_or_init() -> Self {
        match std::fs::read_to_string("xkcd_index.storage") {
            Ok(data) => {
                serde_json::from_str::<Self>(&data).expect("could not deserialize xkcd index state")
            }
            Err(e) => {
                warn!("Xkcd index loading error: {}", e);
                Self { indexed: 0 }
            }
        }
    }

    fn save(&self) {
        let data = serde_json::to_string(self).expect("could not serialize xkcd index state");
        std::fs::write("xkcd_index.storage", data).expect("could not write optout state to file");
    }
}

#[derive(Serialize, Deserialize)]
struct OptOutStore {
    pub set: HashSet<u64>,
}

impl OptOutStore {
    fn load_or_init() -> Self {
        match std::fs::read_to_string("opt_out.storage") {
            Ok(data) => {
                serde_json::from_str::<Self>(&data).expect("could not deserialize rules state")
            }
            Err(e) => {
                warn!("OptOut loading error: {}", e);
                Self {
                    set: HashSet::new(),
                }
            }
        }
    }

    fn save(&self) {
        let data = serde_json::to_string(self).expect("could not serialize optout state");
        std::fs::write("opt_out.storage", data).expect("could not write optout state to file");
    }
}

static XKCD_INDEX: OnceCell<tantivy::Index> = OnceCell::new();
static XKCD_INDEX_READER: OnceCell<tantivy::IndexReader> = OnceCell::new();
static XKCD_INDEX_SCHEMA: OnceCell<tantivy::schema::Schema> = OnceCell::new();
static MESSAGE_REGEX: OnceCell<regex::Regex> = OnceCell::new();

#[help]
#[individual_command_tip = "If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = "-"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Hide"]
#[wrong_channel = "Strike"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    std::mem::drop(
        help_commands::with_embeds(context, msg, args, help_options, groups, owners).await,
    );
    Ok(())
}

#[hook]
async fn before(_ctx: &Context, msg: &Message, command_name: &str) -> bool {
    debug!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    true
}

#[hook]
async fn after(_ctx: &Context, _msg: &Message, command_name: &str, command_result: CommandResult) {
    match command_result {
        Ok(()) => debug!("Processed command '{}'", command_name),
        Err(why) => error!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &Context, _msg: &Message, unknown_command_name: &str) {
    debug!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: &Context, msg: &Message) {
    trace!("Message is not a command '{}'", msg.content);
}

#[hook]
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError, _command_name: &str) {
    match error {
        DispatchError::Ratelimited(info) => {
            std::mem::drop(
                msg.channel_id
                    .say(
                        &ctx.http,
                        &format!(
                            "Try again in {}",
                            util::humanize_duration(
                                &chrono::Duration::from_std(info.rate_limit)
                                    .unwrap_or_else(|_| chrono::Duration::zero())
                            )
                        ),
                    )
                    .await,
            );
        }
        e => trace!(dispatch_error = ?e, "Dispatch error"),
    }
}

#[tokio::main]
async fn main() {
    let config: config::Config = toml::from_str(
        &tokio::fs::read_to_string("config.toml")
            .await
            .expect("Could not load config file"),
    )
    .expect("Failed to parse config");

    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    let http = Http::new_with_token(&config.discord_token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let mut framework = StandardFramework::new()
        .configure(|c| {
            c.with_whitespace(true)
                .on_mention(Some(bot_id))
                .prefix("$")
                .delimiter(' ')
                .owners(owners)
        })
        .before(before)
        .after(after)
        .unrecognised_command(unknown_command)
        .normal_message(normal_message)
        .on_dispatch_error(dispatch_error)
        .help(&MY_HELP);

    for bucket in &config.buckets {
        framework = framework
            .bucket(&bucket.name, |mut b| {
                if let Some(delay) = bucket.delay {
                    b = b.delay(delay);
                }
                if let Some(time_span) = bucket.time_span {
                    b = b.time_span(time_span);
                }
                if let Some(limit) = bucket.limit {
                    b = b.limit(limit);
                }

                b
            })
            .await;
    }

    framework = framework
        .group(&commands::groups::general::GENERAL_GROUP)
        .group(&commands::groups::config::CONFIG_GROUP)
        .group(&commands::groups::greenbook::GREENBOOK_GROUP)
        // .group(&commands::groups::rules::RULES_GROUP)
        .group(&commands::groups::account::ACCOUNT_GROUP)
        .group(&commands::groups::moderation::MODERATION_GROUP)
        .group(&commands::groups::misc::MISC_GROUP)
        .group(&commands::groups::lastfm::LASTFM_GROUP);
    debug!("Framework created");

    let mut client = Client::builder(&config.discord_token)
        .cache_update_timeout(std::time::Duration::from_millis(500))
        .event_handler(handler::Handler)
        .application_id(config.application_id)
        .framework(framework)
        .intents(GatewayIntents::all())
        .await
        .expect("Err creating client");

    let rules_state = Arc::new(Mutex::new(self::rules::State::load()));

    let pool = PgPoolOptions::new()
        .max_connections(config.db_pool_max_size)
        .connect(&config.db_url)
        .await
        .expect("Could not setup db pool");

    let opt_out = Arc::new(Mutex::new(OptOutStore::load_or_init()));

    startup::init_xkcd(&config).await;

    MESSAGE_REGEX
        .set(
            regex::Regex::new(
                r#"https://(?:discord.com|discordapp.com)/channels/(\d+)/(\d+)/(\d+)"#,
            )
            .expect("Could not compile quote link regex"),
        )
        .unwrap();

    {
        let mut data = client.data.write().await;

        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<RulesState>(rules_state);
        data.insert::<OptOut>(Arc::clone(&opt_out));
        data.insert::<DatabasePool>(pool);
        data.insert::<ReqwestClient>(reqwest::Client::new());
        data.insert::<RunningState>(BotState {
            running_since: std::time::Instant::now(),
        });
        data.insert::<XkcdState>(XkcdIndexStorage::load_or_init());
        data.insert::<Config>(config);
    }

    startup::init(&client).await;

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
