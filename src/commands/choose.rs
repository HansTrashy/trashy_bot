use rand::prelude::*;
// use rand::seq::SliceRandom;
use crate::util::sanitize_for_other_bot_commands;
use serenity::utils::{content_safe, ContentSafeOptions};
use serenity::{
    framework::standard::{
        Args, CommandResult,
        macros::command,
    },
    model::channel::Message,
};
use serenity::prelude::*;
use log::*;

#[command]
pub fn choose(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut rng = rand::thread_rng();

    let settings = ContentSafeOptions::default().clean_channel(false);

    if args.len() < 2 {
        return match msg
            .channel_id
            .say(&ctx.http, "You have to give at least 2 options")
        {
            Ok(_msg) => Ok(()),
            Err(e) => {
                error!("Failure sending message: {:?}", e);
                Err(e.into())
            }
        };
    }

    let chosen = args.iter::<String>().choose(&mut rng).unwrap().unwrap();

    match msg.channel_id.say(
        &ctx.http,
        content_safe(
            &ctx.cache,
            &sanitize_for_other_bot_commands(&chosen),
            &settings,
        ),
    ) {
        Ok(_msg) => Ok(()),
        Err(e) => {
            error!("Failure sending message: {:?}", e);
            Err(e.into())
        }
    }
}
