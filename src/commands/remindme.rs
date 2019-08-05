use serde_derive::Deserialize;
use serenity::{
    prelude::*,
    framework::standard::{Args, CommandResult, macros::command},
    http::Http,
    model::prelude::*,
};
use log::*;
use white_rabbit::{Utc, Scheduler, DateResult, Duration};
use crate::{DispatcherKey, SchedulerKey};
use crate::dispatch::DispatchEvent;
use hey_listen::sync::ParallelDispatcherRequest as DispatcherRequest;
use std::sync::Arc;
use serenity::utils::{content_safe, ContentSafeOptions};
use crate::util;

#[command]
#[description = "Reminds you after the given time with the given text. Allowed time units: s,m,h,d."]
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
                    .get_mut::<SchedulerKey>()
                    .expect("expected scheduler")
                    .clone()
            };

            // let dispatcher = {
            //     let mut context = ctx.data.write();
            //     context
            //         .get_mut::<DispatcherKey>()
            //         .expect("expected dispatcher")
            //         .clone()
            // };

            let http = ctx.http.clone();
            let cache = ctx.cache.clone();
            let _ = msg.react(&ctx, ReactionType::Unicode("✅".to_string()));
            let msg = msg.clone();

            let mut scheduler = scheduler.write();

            scheduler.add_task_duration(duration, move |_| {
                let bot_msg = match msg.reply((&cache, &*http), &args) {
                    Ok(msg) => msg,
                    Err(why) => {
                        error!("Could not send message: {:?}", why);
                        return DateResult::Done;
                    }
                };

                // let http = http.clone();
                // dispatcher.write().add_fn(
                //     DispatchEvent::ReactEvent(bot_msg.id, msg.author.id),
                //     Box::new(move |_| {
                //         if let Err(why) = bot_msg.channel_id.say(&http, "Thanks for reacting!") {
                //             error!("Could not send message: {:?}", why);
                //         }
                //         Some(DispatcherRequest::StopListening)
                //     }),
                // );

                DateResult::Done
            });

            Ok(())
        }
    }
}
