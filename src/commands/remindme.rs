use serde::Deserialize;
use serenity::{
    prelude::*,
    framework::standard::{Args, CommandResult, macros::command},
    http::Http,
    model::prelude::*,
};
use log::*;
use crate::DispatcherKey;
use crate::dispatch::DispatchEvent;
use hey_listen::sync::ParallelDispatcherRequest as DispatcherRequest;
use std::sync::Arc;
use serenity::utils::{content_safe, ContentSafeOptions};
use crate::util;
use crate::TrashyScheduler;
use crate::scheduler::Task;
use time::Duration;

#[command]
#[description = "Reminds you after the given time with the given text. Allows (w, d, h, m, s)"]
#[example("15m Pizza ist fertig!")]
#[usage("*duration* *message*")]
#[min_args(1)]
fn remindme(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let duration = util::parse_duration(&args.single::<String>()?);

    match duration {
        None => {
            let _ = msg.reply(&ctx, "Unknown time unit. Allowed units are: s,m,h,d,w");
            Ok(())
        }
        Some(duration) => {
            let args = content_safe(
                &ctx,
                &args.rest().to_string(),
                &ContentSafeOptions::default(),
            );

            let scheduler = {
                let mut context = ctx.data.write();
                context
                    .get_mut::<TrashyScheduler>()
                    .expect("could not get scheduler")
                    .clone()
            };

            let http = ctx.http.clone();
            let cache = ctx.cache.clone();
            let _ = msg.react(&ctx, ReactionType::Unicode("✅".to_string()));
            let msg = msg.clone();

            let task = Task::reply(*msg.author.id.as_u64(), *msg.channel_id.as_u64(), args);
            scheduler.add_task(duration, task);

            Ok(())
        }
    }
}
