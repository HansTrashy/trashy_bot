#![deny(clippy::all)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![deny(nonstandard_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(unused)]
//! Trashy Bot

#[macro_use]
extern crate tantivy;

use deadpool_postgres::Pool;
use dotenv::dotenv;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serenity::{
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
use std::collections::HashSet;
use std::{env, sync::Arc};
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument, trace, warn, Level};

mod commands;
mod handler;
mod migrations;
mod models;
mod reaction_roles;
mod rules;
mod startup;
mod util;

struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct DatabasePool;
impl TypeMapKey for DatabasePool {
    type Value = Pool;
}

struct ReactionRolesState;
impl TypeMapKey for ReactionRolesState {
    type Value = Arc<Mutex<self::reaction_roles::State>>;
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
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

lazy_static! {
    pub static ref LASTFM_API_KEY: String =
        env::var("LASTFM_API_KEY").expect("Expected a lastfm token in the environment");
}

lazy_static! {
    pub static ref XKCD_INDEX_PATH: std::path::PathBuf = std::path::PathBuf::from(
        env::var("XKCD_INDEX").expect("Expected a xkcd index path in the environment")
    );
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
async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    match error {
        DispatchError::Ratelimited(duration) => {
            let _ = msg
                .channel_id
                .say(
                    &ctx.http,
                    &format!(
                        "Try again in {}",
                        util::humanize_duration(
                            &chrono::Duration::from_std(duration)
                                .unwrap_or(chrono::Duration::zero())
                        )
                    ),
                )
                .await;
        }
        e => trace!(dispatch_error = ?e, "Dispatch error"),
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a discord token in the environment");
    let http = Http::new_with_token(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not access application info: {:?}", why),
    };

    let framework = StandardFramework::new()
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
        .help(&MY_HELP)
        .bucket("slotmachine", |b| b.delay(10))
        .await
        .bucket("blackjack", |b| b.delay(600))
        .await
        .bucket("lastfm", |b| b.delay(1).time_span(10).limit(5))
        .await
        .group(&commands::groups::general::GENERAL_GROUP)
        .group(&commands::groups::config::CONFIG_GROUP)
        .group(&commands::groups::greenbook::GREENBOOK_GROUP)
        // .group(&commands::groups::rules::RULES_GROUP)
        .group(&commands::groups::roles::ROLES_GROUP)
        .group(&commands::groups::account::ACCOUNT_GROUP)
        .group(&commands::groups::moderation::MODERATION_GROUP)
        .group(&commands::groups::misc::MISC_GROUP)
        .group(&commands::groups::lastfm::LASTFM_GROUP);
    debug!("Framework created");

    let mut client = Client::new(&token)
        .cache_update_timeout(std::time::Duration::from_millis(500))
        .event_handler(handler::Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    let rr_state = Arc::new(Mutex::new(self::reaction_roles::State::load_set()));
    let rules_state = Arc::new(Mutex::new(self::rules::State::load()));

    let async_db_pool = deadpool_postgres::Config::from_env("PG")
        .expect("PG env vars not found")
        .create_pool(tokio_postgres::NoTls)
        .expect("could not create async db pool");

    {
        let mut db_client = async_db_pool.get().await.unwrap();
        migrations::run(&mut db_client)
            .await
            .expect("could not run migrations");
    }
    debug!("Database pool created");

    let opt_out = Arc::new(Mutex::new(OptOutStore::load_or_init()));

    {
        let mut data = client.data.write().await;

        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<ReactionRolesState>(rr_state);
        data.insert::<RulesState>(rules_state);
        data.insert::<OptOut>(Arc::clone(&opt_out));
        data.insert::<DatabasePool>(async_db_pool);
        data.insert::<ReqwestClient>(reqwest::Client::new());
        data.insert::<RunningState>(BotState {
            running_since: std::time::Instant::now(),
        });
        data.insert::<XkcdState>(XkcdIndexStorage::load_or_init());
    }

    startup::on_startup(&client).await;

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}

#[cfg(test)]
mod tests {}
