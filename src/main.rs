#[macro_use]
extern crate serenity;
#[macro_use]
extern crate diesel;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use log::{debug, error, info, trace, warn};
use serenity::{
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        help_commands, Args, CommandOptions, DispatchError, HelpBehaviour, StandardFramework,
    },
    model::{
        channel::Message, channel::Reaction, channel::ReactionType, gateway::Ready, Permissions,
    },
    prelude::*,
    utils::{content_safe, ContentSafeOptions},
};
use std::{collections::HashMap, env, fmt::Write, sync::Arc};

// This imports `typemap`'s `Key` as `TypeMapKey`.
use serenity::prelude::*;

mod commands;
mod handler;
mod interaction;
mod logger;
mod models;
mod schema;

struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct DatabaseConnection;

impl TypeMapKey for DatabaseConnection {
    type Value = Arc<Mutex<PgConnection>>;
}

struct Waiter;

impl TypeMapKey for Waiter {
    type Value = Arc<Mutex<self::interaction::wait::Wait>>;
}

fn main() {
    // load .env file
    kankyo::load().expect("no env file");
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let mut client = Client::new(&token, handler::Handler).expect("Err creating client");

    let conn = Arc::new(Mutex::new(
        PgConnection::establish(
            &env::var("DATABASE_URL").expect("Expected a database in the environment"),
        )
        .expect("Error connecting to database"),
    ));

    let waiter = Arc::new(Mutex::new(self::interaction::wait::Wait::new()));

    {
        let mut data = client.data.lock();
        data.insert::<DatabaseConnection>(conn);
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
        data.insert::<Waiter>(waiter);
    }

    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.allow_whitespace(true)
                    .on_mention(true)
                    .prefixes(vec![".", "$", "&"])
                    .prefix_only_cmd(commands::about::about)
                    .delimiter(" ")
            })
            .before(|_ctx, msg, command_name| {
                println!(
                    "Got command '{}' by user '{}'",
                    command_name, msg.author.name
                );

                true
            })
            .after(|_, _, command_name, error| match error {
                Ok(()) => println!("Processed command '{}'", command_name),
                Err(why) => println!("Command '{}' returned error {:?}", command_name, why),
            })
            .unrecognised_command(|_, _, unknown_command_name| {
                debug!("Could not find command named '{}'", unknown_command_name);
            })
            .message_without_command(|_, message| {
                debug!("Message is not a command '{}'", message.content);
            })
            .on_dispatch_error(|_ctx, msg, error| {
                if let DispatchError::RateLimited(seconds) = error {
                    let _ = msg
                        .channel_id
                        .say(&format!("Try this again in {} seconds.", seconds));
                }
            })
            .command("about", |c| c.cmd(commands::about::about))
            .command("roll", |c| c.cmd(commands::roll::roll))
            .command("choose", |c| c.cmd(commands::choose::choose))
            .command("fav", |c| c.cmd(commands::fav::fav))
            .command("kick", |c| c.check(admin_check).cmd(commands::kick::kick))
            .command("ban", |c| c.check(admin_check).cmd(commands::ban::ban))
            .command("quote", |c| c.cmd(commands::quote::quote))
            .command("untagged", |c| c.cmd(commands::fav::untagged))
            .command("bank", |c| c.cmd(commands::bank::bank))
            .command("payday", |c| c.cmd(commands::bank::payday))
            .command("slot", |c| c.cmd(commands::bank::slot))
            .command("leaderboard", |c| c.cmd(commands::bank::leaderboard))
            .command("transfer", |c| c.cmd(commands::bank::transfer))
            .help(help_commands::with_embeds),
    );

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

fn admin_check(_: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> bool {
    if let Some(member) = msg.member() {
        if let Ok(permissions) = member.permissions() {
            return permissions.administrator();
        }
    }
    false
}

#[cfg(test)]
mod tests {}
