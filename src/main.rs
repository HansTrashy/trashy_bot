#![deny(clippy::all)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::non_ascii_literal)]
#![deny(nonstandard_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(unused)]
#![feature(async_closure)]
//! Trashy Bot

#[macro_use]
extern crate serenity;
#[macro_use]
extern crate diesel;

use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use hey_listen::sync::ParallelDispatcher as Dispatcher;
use lazy_static::lazy_static;
use log::*;
use serde::{Deserialize, Serialize};
use serenity::{
    client::bridge::gateway::ShardManager,
    client::bridge::voice::ClientVoiceManager,
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
    voice,
};
use std::collections::HashSet;
use std::{env, sync::Arc};

mod blackjack;
mod commands;
mod dispatch;
mod handler;
mod interaction;
mod logger;
mod models;
mod reaction_roles;
mod rules;
mod scheduler;
mod schema;
mod util;

struct ShardManagerContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct DatabaseConnection;
impl TypeMapKey for DatabaseConnection {
    type Value = Pool<ConnectionManager<PgConnection>>;
}

struct DispatcherKey;
impl TypeMapKey for DispatcherKey {
    type Value = Arc<RwLock<Dispatcher<crate::dispatch::DispatchEvent>>>;
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

struct BlackjackState;
impl TypeMapKey for BlackjackState {
    type Value = Arc<Mutex<self::blackjack::State>>;
}

struct VoiceManager;
impl TypeMapKey for VoiceManager {
    type Value = Arc<Mutex<ClientVoiceManager>>;
}

struct TrashyScheduler;
impl TypeMapKey for TrashyScheduler {
    type Value = Arc<scheduler::Scheduler>;
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
// This replaces the information that a user can pass
// a command-name as argument to gain specific information about it.
#[individual_command_tip = "If you want more information about a specific command, just pass the command as argument."]
// Some arguments require a `{}` in order to replace it with contextual information.
// In this case our `{}` refers to a command's name.
#[command_not_found_text = "Could not find: `{}`."]
// Define the maximum Levenshtein-distance between a searched command-name
// and commands. If the distance is lower than or equal the set distance,
// it will be displayed as a suggestion.
// Setting the distance to 0 will disable suggestions.
#[max_levenshtein_distance(3)]
// When you use sub-groups, Serenity will use the `indention_prefix` to indicate
// how deeply an item is indented.
// The default value is "-", it will be changed to "+".
#[indention_prefix = "-"]
// On another note, you can set up the help-menu-filter-behaviour.
// Here are all possible settings shown on all possible options.
// First case is if a user lacks permissions for a command, we can hide the command.
#[lacking_permissions = "Hide"]
// If the user is nothing but lacking a certain role, we just display it hence our variant is `Nothing`.
#[lacking_role = "Hide"]
// The last `enum`-variant is `Strike`, which ~~strikes~~ a command.
#[wrong_channel = "Strike"]
// Serenity will automatically analyse and generate a hint/tip explaining the possible
// cases of ~~strikethrough-commands~~, but only if
// `strikethrough_commands_tip(Some(""))` keeps `Some()` wrapping an empty `String`, which is the default value.
// If the `String` is not empty, your given `String` will be used instead.
// If you pass in a `None`, no hint will be displayed at all.
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
    kankyo::load().expect("no env file");
    // setup logging
    logger::setup().expect("Could not setup logging");
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a discord token in the environment");
    let mut client = Client::new(&token, handler::Handler).expect("Err creating client");

    let db_manager = ConnectionManager::<PgConnection>::new(
        env::var("DATABASE_URL").expect("Expected a database in the environment"),
    );
    let db_pool = Pool::builder()
        .build(db_manager)
        .expect("Failed to create db pool.");

    let waiter = Arc::new(Mutex::new(self::interaction::wait::Wait::new()));
    let rr_state = Arc::new(Mutex::new(self::reaction_roles::State::load_set()));
    let rules_state = Arc::new(Mutex::new(self::rules::State::load()));
    let blackjack_state = Arc::new(Mutex::new(self::blackjack::State::load(db_pool.clone())));

    let mut dispatcher: Dispatcher<crate::dispatch::DispatchEvent> = Dispatcher::default();
    dispatcher
        .num_threads(2)
        .expect("could not construct threadpool for dispatcher");

    let trashy_scheduler = Arc::new(scheduler::Scheduler::new(
        Arc::clone(&client.cache_and_http),
        db_pool.clone(),
    ));

    let opt_out = Arc::new(Mutex::new(OptOutStore::load_or_init()));

    {
        let mut data = client.data.write();

        data.insert::<DatabaseConnection>(db_pool);
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<Waiter>(waiter);
        data.insert::<ReactionRolesState>(rr_state);
        data.insert::<RulesState>(rules_state);
        data.insert::<BlackjackState>(blackjack_state);
        data.insert::<DispatcherKey>(Arc::new(RwLock::new(dispatcher)));
        data.insert::<VoiceManager>(Arc::clone(&client.voice_manager));
        data.insert::<TrashyScheduler>(Arc::clone(&trashy_scheduler));
        data.insert::<OptOut>(Arc::clone(&opt_out));
    }

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
                    .delimiter(" ")
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
            .group(&commands::groups::reaction_roles::REACTION_ROLES_GROUP)
            .group(&commands::groups::account::ACCOUNT_GROUP)
            .group(&commands::groups::moderation::MODERATION_GROUP)
            .group(&commands::groups::misc::MISC_GROUP)
            .group(&commands::groups::lastfm::LASTFM_GROUP)
            .group(&commands::groups::voice::VOICE_GROUP),
    );

    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}

#[check]
#[name = "Owner"]
fn owner_check(_: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    (msg.author.id == 179_680_865_805_271_040).into()
}

#[check]
#[name = "Admin"]
fn admin_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    if let Some(member) = msg.member(&ctx.cache) {
        if let Ok(permissions) = member.permissions(&ctx.cache) {
            return permissions.administrator().into();
        }
    }
    false.into()
}

#[cfg(test)]
mod tests {}
