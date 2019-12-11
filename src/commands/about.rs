use log::*;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command]
#[description = "Info about the bot"]
#[num_args(0)]
fn about(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    match msg.channel_id.say(
        &ctx.http,
        "Der mülligste aller Bots!\nSource: https://github.com/HansTrashy/trashy_bot",
    ) {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failure sending about message: {:?}", e);
            Err(e.into())
        }
    }
}
