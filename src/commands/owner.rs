use serenity::{
    framework::standard::{
        Args, CommandResult,
        macros::command,
    },
    model::channel::Message,
};
use serenity::model::gateway::Activity;
use serenity::model::user::OnlineStatus;
use serenity::prelude::*;
use log::*;

#[command]
#[allowed_roles("Mods")]
pub fn setstatus(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    ctx.set_presence(Some(Activity::listening("$help")), OnlineStatus::Online);
    Ok(())
}
