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
#[description = "Info about the bot"]
fn about(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    match msg.channel_id.say(
        &ctx.http,
        "Der mülligste aller Bots!\nSource: https://github.com/HansTrashy/trashy_bot",
    ) {
        Ok(_msg) => Ok(()),
        Err(e) => {
            error!("Failure sending about message: {:?}", e);
            Err(e.into())
        }
    }
}
