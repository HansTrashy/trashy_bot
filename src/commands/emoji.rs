use itertools::Itertools;
use log::*;
use serenity::model::channel::ReactionType;
use serenity::prelude::*;
use serenity::utils::{content_safe, ContentSafeOptions};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use std::iter::FromIterator;

#[command]
#[description = "Let the bot post an Emoji"]
#[num_args(1)]
#[only_in("guilds")]
fn emoji(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let emoji_name = args.rest();

    if let Some(guild) = msg.guild(&ctx) {
        for (_id, e) in guild.read().emojis.iter() {
            if e.name == emoji_name {
                let _ = msg
                    .channel_id
                    .send_message(&ctx, |m| m.content(format!("{}", e)));
                return Ok(());
            }
        }
    }

    let _ = msg.channel_id.send_message(&ctx, |m| {
        m.content("Could not find the emoji you are looking for!")
    });

    Ok(())
}
