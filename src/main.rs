#![deny(clippy::all)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![deny(nonstandard_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(unused)]
//! Trashy Bot

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
use tracing_log::LogTracer;

mod commands;
mod dispatch;
mod handler;
mod interaction;
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

struct DatabasePool;
impl TypeMapKey for DatabasePool {
    type Value = Pool;
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
    type Value = Arc<Mutex<dispatch::Dispatcher>>;
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
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await
}

lazy_static! {
    pub static ref LASTFM_API_KEY: String =
        env::var("LASTFM_API_KEY").expect("Expected a lastfm token in the environment");
}

#[hook]
async fn before(_ctx: &mut Context, msg: &Message, command_name: &str) -> bool {
    debug!(
        "Got command '{}' by user '{}'",
        command_name, msg.author.name
    );

    true
}

#[hook]
async fn after(
    _ctx: &mut Context,
    _msg: &Message,
    command_name: &str,
    command_result: CommandResult,
) {
    match command_result {
        Ok(()) => debug!("Processed command '{}'", command_name),
        Err(why) => debug!("Command '{}' returned error {:?}", command_name, why),
    }
}

#[hook]
async fn unknown_command(_ctx: &mut Context, _msg: &Message, unknown_command_name: &str) {
    debug!("Could not find command named '{}'", unknown_command_name);
}

#[hook]
async fn normal_message(_ctx: &mut Context, msg: &Message) {
    trace!("Message is not a command '{}'", msg.content);
}

#[hook]
async fn dispatch_error(ctx: &mut Context, msg: &Message, error: DispatchError) -> () {
    if let DispatchError::Ratelimited(seconds) = error {
        let _ = msg
            .channel_id
            .say(&ctx.http, &format!("Try again in {} seconds", seconds))
            .await;
    } else {
        let _ = msg
            .channel_id
            .say(
                &ctx.http,
                &format!("Something didn't quite work: {:?}", error),
            )
            .await;
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    LogTracer::init().expect("could not setup Log tracer");

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

    let mut client = Client::new_with_framework(&token, handler::Handler, framework)
        .await
        .expect("Err creating client");
    debug!("Client created");

    let waiter = Arc::new(Mutex::new(self::interaction::wait::Wait::new()));
    let rr_state = Arc::new(Mutex::new(self::reaction_roles::State::load_set()));
    let rules_state = Arc::new(Mutex::new(self::rules::State::load()));

    let async_db_pool = deadpool_postgres::Config::from_env("PG")
        .expect("PG env vars not found")
        .create_pool(tokio_postgres::NoTls)
        .expect("could not create async db pool");

    {
        let mut client = async_db_pool.get().await.unwrap();
        migrations::run(&mut client)
            .await
            .expect("could not run migrations");
    }
    debug!("Database pool created");

    let rt = Arc::new(tokio::runtime::Handle::current());
    let trashy_scheduler = Arc::new(scheduler::Scheduler::new(
        Arc::clone(&rt),
        Arc::clone(&client.cache_and_http),
        async_db_pool.clone(),
    ));

    let opt_out = Arc::new(Mutex::new(OptOutStore::load_or_init()));

    let trashy_dispatcher = Arc::new(Mutex::new(dispatch::Dispatcher::new()));

    let trashy_dispatcher_clone = Arc::clone(&trashy_dispatcher);
    tokio::spawn(async move {
        loop {
            tokio::time::delay_for(std::time::Duration::from_secs(60)).await;
            trashy_dispatcher_clone.lock().await.check_expiration();
        }
    });

    {
        let mut data = client.data.write().await;

        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<Waiter>(waiter);
        data.insert::<ReactionRolesState>(rr_state);
        data.insert::<RulesState>(rules_state);
        data.insert::<TrashyScheduler>(Arc::clone(&trashy_scheduler));
        data.insert::<OptOut>(Arc::clone(&opt_out));
        data.insert::<DatabasePool>(async_db_pool);
        data.insert::<TrashyDispatcher>(trashy_dispatcher);
    }

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}

#[cfg(test)]
mod tests {}
