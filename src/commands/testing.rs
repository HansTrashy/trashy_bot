use crate::new_dispatch::{Dispatcher, Listener};
use chrono::prelude::*;
use chrono::{DateTime, Utc};
use log::*;
use serde::Deserialize;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
};

#[command]
#[description = "Create a dispatcher for the given emoji"]
pub fn dispatch(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let emoji = ReactionType::from(args.rest());

    let mut data = ctx.data.write();
    let dispatcher = {
        data.get_mut::<crate::TrashyDispatcher>()
            .expect("Expected Dispatcher.")
            .clone()
    };

    let (ctx_1, ctx_2) = (ctx.cache.clone(), ctx.http.clone());
    let msg_clone = msg.clone();
    dispatcher.lock().add_listener(
        crate::new_dispatch::DispatchEvent::ReactEvent(msg.id, emoji, msg.channel_id, msg.author.id),
        Listener::new(
            std::time::Duration::from_secs(10),
            Box::new(move || {
                let _ = msg_clone.reply((&ctx_1, &*ctx_2), "When life gives you lemons...");
            }),
        ),
    );

    Ok(())
}
