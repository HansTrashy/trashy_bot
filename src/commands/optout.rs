use crate::OptOut;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command]
#[description = "Opt out of the fav/quote features"]
pub async fn optout(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let opt_out = match ctx.data.read().await.get::<OptOut>() {
        Some(v) => v.clone(),
        None => {
            std::mem::drop(msg.reply(ctx, "OptOut list not available"));
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
