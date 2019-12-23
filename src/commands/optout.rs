use crate::OptOut;
use log::*;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command]
#[description = "Opt out of the fav/quote features"]
pub fn optout(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read();
    let opt_out = match data.get::<OptOut>() {
        Some(v) => v,
        None => {
            let _ = msg.reply(&ctx, "OptOut list not available");
            panic!("no optout");
        }
    };

    let mut lock = opt_out.lock();
    let id = *msg.author.id.as_u64();

    if !lock.set.insert(id) {
        lock.set.remove(&id);
    }

    lock.save();

    Ok(())
}
