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
#[description = "Posts the Max Goldt Quote"]
#[example("")]
fn goldt(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let content = r#"Diese Zeitung ist ein Organ der Niedertracht. Es ist falsch, sie zu lesen. Jemand, der zu dieser Zeitung beiträgt, ist gesellschaftlich absolut inakzeptabel. Es wäre verfehlt, zu einem ihrer Redakteure freundlich oder auch nur höflich zu sein. Man muß so unfreundlich zu ihnen sein, wie es das Gesetz gerade noch zuläßt. Es sind schlechte Menschen, die Falsches tun. - *Max Goldt*"#;

    let _ = msg.channel_id.send_message(&ctx, |m| m.content(content));

    Ok(())
}
