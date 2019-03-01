#[macro_use]
extern crate serenity;
#[macro_use]
extern crate diesel;

use std::{collections::HashMap, env, fmt::Write, sync::Arc};

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

// This imports `typemap`'s `Key` as `TypeMapKey`.
use serenity::prelude::*;

mod commands;
mod handler;
mod models;
mod schema;

fn main() {
    // load .env file
    kankyo::load().expect("no env file");
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let mut client = Client::new(&token, handler::Handler).expect("Err creating client");

    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.allow_whitespace(true)
                    .on_mention(true)
                    .prefix("~")
                    .prefix_only_cmd(commands::about::about)
                    .delimiter(" ")
            })
            .before(|ctx, msg, command_name| {
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
                println!("Could not find command named '{}'", unknown_command_name);
            })
            .message_without_command(|_, message| {
                println!("Message is not a command '{}'", message.content);
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
            .help(help_commands::with_embeds),
    );

    if let Err(why) = client.start() {
        println!("Client error: {:?}", why);
    }
}

#[cfg(test)]
mod tests {}
