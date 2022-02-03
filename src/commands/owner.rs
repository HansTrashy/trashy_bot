use crate::util;
use crate::OptOut;
use serenity::futures::stream::StreamExt;
use serenity::model::channel::Attachment;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
};
use std::time::Duration;
use tracing::{debug, trace};

#[command]
#[description = "leave the server"]
#[owners_only]
#[only_in("guilds")]
pub async fn leave(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if let Some(guild_id) = msg.guild_id {
        guild_id.leave(ctx).await?;
    }

    Ok(())
}
