#![deny(clippy::all)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
// #![allow(clippy::non_ascii_literal)]
#![deny(nonstandard_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(unused)]
//! Trashy Bot

use crate::dispatch::Dispatcher;
use dotenv::dotenv;
use lazy_static::lazy_static;
use log::*;
use postgres::NoTls;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use serde::{Deserialize, Serialize};
use serenity::{
    client::bridge::gateway::ShardManager,
    client::Context,
    framework::standard::{
        help_commands,
        macros::{check, command, group, help},
        Args, CheckResult, CommandGroup, CommandOptions, CommandResult, DispatchError, HelpOptions,
        StandardFramework,
    },
    model::{
        channel::{Channel, Message},
        gateway::Ready,
        id::UserId,
    },
    prelude::*,
};
use std::collections::HashSet;
use std::{env, sync::Arc};

mod commands;
mod dispatch;
mod handler;
mod interaction;
mod logger;
mod migrations;
mod models;
mod reaction_roles;
mod rules;
mod scheduler;
mod util;

struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct DatabaseConnection;
impl TypeMapKey for DatabaseConnection {
    type Value = Pool<PostgresConnectionManager<NoTls>>;
}

struct Waiter;
impl TypeMapKey for Waiter {
    type Value = Arc<Mutex<self::interaction::wait::Wait>>;
}

struct ReactionRolesState;
impl TypeMapKey for ReactionRolesState {
    type Value = Arc<Mutex<self::reaction_roles::State>>;
}

struct RulesState;
impl TypeMapKey for RulesState {
    type Value = Arc<Mutex<self::rules::State>>;
}

struct TrashyScheduler;
impl TypeMapKey for TrashyScheduler {
    type Value = Arc<scheduler::Scheduler>;
}

struct TrashyDispatcher;
impl TypeMapKey for TrashyDispatcher {
    type Value = Arc<Mutex<Dispatcher>>;
}

struct OptOut;
impl TypeMapKey for OptOut {
    type Value = Arc<Mutex<OptOutStore>>;
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
                warn!("OptOutp loading error: {}", e);
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
fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

lazy_static! {
    pub static ref LASTFM_API_KEY: String =
        env::var("LASTFM_API_KEY").expect("Expected a lastfm token in the environment");
}

fn main() {
    // load .env file
    dotenv().ok();
    // setup logging
    logger::setup().expect("Could not setup logging");
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a discord token in the environment");
    let mut client = Client::new(&token, handler::Handler).expect("Err creating client");

    let db_manager = PostgresConnectionManager::new(
        env::var("DATABASE_URL")
            .expect("no database url specified")
            .parse()
            .expect("could not parse DATABASE_URL as PG CONFIG"),
        NoTls,
    );
    let db_pool = r2d2::Pool::new(db_manager).expect("Failed to create db pool.");

    {
        let mut client = db_pool.get().unwrap();
        migrations::run(&mut client).expect("could not run migrations");
    }

    let waiter = Arc::new(Mutex::new(self::interaction::wait::Wait::new()));
    let rr_state = Arc::new(Mutex::new(self::reaction_roles::State::load_set()));
    let rules_state = Arc::new(Mutex::new(self::rules::State::load()));

    let async_db_pool = deadpool_postgres::Config::from_env("PG")
        .expect("PG env vars not found")
        .create_pool(tokio_postgres::NoTls)
        .expect("could not create async db pool");

    let rt = Arc::new(tokio::runtime::Runtime::new().unwrap());
    let trashy_scheduler = Arc::new(scheduler::Scheduler::new(
        Arc::clone(&rt),
        Arc::clone(&client.cache_and_http),
        async_db_pool,
    ));

    let trashy_dispatcher = Arc::new(Mutex::new(Dispatcher::new()));

    let opt_out = Arc::new(Mutex::new(OptOutStore::load_or_init()));

    {
        let mut data = client.data.write();

        data.insert::<DatabaseConnection>(db_pool);
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<Waiter>(waiter);
        data.insert::<ReactionRolesState>(rr_state);
        data.insert::<RulesState>(rules_state);
        data.insert::<TrashyScheduler>(Arc::clone(&trashy_scheduler));
        data.insert::<TrashyDispatcher>(Arc::clone(&trashy_dispatcher));
        data.insert::<OptOut>(Arc::clone(&opt_out));
    }

    // setup interval to check expiration of dispatcher listener
    rt.spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(20));
        loop {
            interval.tick().await;
            trashy_dispatcher.lock().check_expiration();
        }
    });

    let (owners, bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }
        Err(why) => panic!("Could not acces application information: {:?}", why),
    };

    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.with_whitespace(true)
                    .on_mention(Some(bot_id))
                    .prefix("$")
                    .delimiter(' ')
                    .owners(owners)
            })
            .before(|_ctx, msg, command_name| {
                debug!(
                    "Got command '{}' by user '{}'",
                    command_name, msg.author.name
                );

                true
            })
            .after(|_, _, command_name, error| match error {
                Ok(()) => debug!("Processed command '{}'", command_name),
                Err(why) => debug!("Command '{}' returned error {:?}", command_name, why),
            })
            .unrecognised_command(|_, _, unknown_command_name| {
                debug!("Could not find command named '{}'", unknown_command_name);
            })
            .normal_message(|_, message| {
                trace!("Message is not a command '{}'", message.content);
            })
            .on_dispatch_error(|ctx, msg, error| {
                if let DispatchError::Ratelimited(seconds) = error {
                    let _ = msg.channel_id.say(
                        &ctx.http,
                        &format!("Versuche es in {} sekunden noch einmal.", seconds),
                    );
                } else {
                    let _ = msg.channel_id.say(
                        &ctx.http,
                        &format!("Something didn't quite work: {:?}", error),
                    );
                }
            })
            .help(&MY_HELP)
            .bucket("slotmachine", |b| b.delay(10))
            .bucket("blackjack", |b| b.delay(600))
            .bucket("lastfm", |b| b.delay(1).time_span(10).limit(5))
            .group(&commands::groups::general::GENERAL_GROUP)
            .group(&commands::groups::config::CONFIG_GROUP)
            .group(&commands::groups::greenbook::GREENBOOK_GROUP)
            .group(&commands::groups::rules::RULES_GROUP)
            .group(&commands::groups::roles::ROLES_GROUP)
            .group(&commands::groups::account::ACCOUNT_GROUP)
            .group(&commands::groups::moderation::MODERATION_GROUP)
            .group(&commands::groups::misc::MISC_GROUP)
            .group(&commands::groups::lastfm::LASTFM_GROUP),
    );

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}

#[cfg(test)]
mod tests {}
