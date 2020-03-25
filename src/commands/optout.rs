use crate::OptOut;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command]
#[description = "Opt out of the fav/quote features"]
pub async fn optout(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let opt_out = match data.get::<OptOut>() {
        Some(v) => v,
        None => {
            let _ = msg.reply(&ctx, "OptOut list not available");
            panic!("no optout");
        }
    };

    let mut lock = opt_out.lock().await;
    let id = *msg.author.id.as_u64();

    if !lock.set.insert(id) {
        lock.set.remove(&id);
    }

    lock.save();

    Ok(())
}
