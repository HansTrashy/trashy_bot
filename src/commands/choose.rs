use rand::prelude::*;
// use rand::seq::SliceRandom;
use crate::util::sanitize_for_other_bot_commands;
use serenity::prelude::*;
use serenity::utils::{content_safe, ContentSafeOptions};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use tracing::error;

#[command]
#[description = "Choose between things"]
#[aliases("ch00se")]
#[min_args(2)]
pub async fn choose(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let settings = ContentSafeOptions::default().clean_channel(false);

    if args.len() < 2 {
        return match msg
            .channel_id
            .say(&ctx.http, "You have to give at least 2 options")
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failure sending message: {:?}", e);
                Err(e.into())
            }
        };
    }

    let chosen = args
        .iter::<String>()
        .choose(&mut rand::thread_rng())
        .unwrap()
        .unwrap();

    match msg
        .channel_id
        .say(
            &ctx.http,
            content_safe(
                &ctx.cache,
                &sanitize_for_other_bot_commands(&chosen),
                &settings,
            )
            .await,
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failure sending message: {:?}", e);
            Err(e.into())
        }
    }
}
