use crate::util;
use crate::RunningState;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use tracing::error;

#[command]
#[description = "Info about the bot"]
#[num_args(0)]
async fn about(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let running_since = ctx
        .data
        .read()
        .await
        .get::<RunningState>()
        .ok_or("Failed to acces RunningState")?
        .running_since;

    match msg
        .channel_id
        .say(
            ctx,
            format!("A really trashy bot!\nRunning for {}.\nSource: https://github.com/HansTrashy/trashy_bot",
             util::humanize_duration(&chrono::Duration::from_std(running_since.elapsed()).unwrap_or(chrono::Duration::zero()))),
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failure sending about message: {:?}", e);
            Err(e.into())
        }
    }
}
