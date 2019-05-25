use lazy_static::lazy_static;
use log::*;
use regex::Regex;
use serenity::model::channel::Attachment;
use serenity::model::id::ChannelId;
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
pub fn quote(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    lazy_static! {
        static ref QUOTE_LINK_REGEX: Regex = Regex::new(r#"https://discordapp.com/channels/\d+/(\d+)/(\d+)"#)
            .expect("couldnt compile quote link regex");
    }
    for caps in QUOTE_LINK_REGEX.captures_iter(&args.rest()) {
        let quote_channel_id = caps[1].parse::<u64>()?;
        let quote_msg_id = caps[2].parse::<u64>()?;

        if let Ok(quoted_msg) = ChannelId(quote_channel_id).message(&ctx.http, quote_msg_id) {
            if let Some(image) = quoted_msg.attachments.iter().cloned().filter(|a| a.width.is_some()).collect::<Vec<Attachment>>().first() {
                let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e|
                    e.author(|a| a.name(&quoted_msg.author.name).icon_url(&quoted_msg.author.static_avatar_url().unwrap_or_default()))
                    .color((0,120,220))
                    .description(&quoted_msg.content)
                    .image(&image.url)
                    .footer(|f| f.text(&format!("{} | Zitiert von: {}", &quoted_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"), &msg.author.name)))
                )});
            } else {
                let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e|
                    e.author(|a| a.name(&quoted_msg.author.name).icon_url(&quoted_msg.author.static_avatar_url().unwrap_or_default()))
                    .color((0,120,220))
                    .description(&quoted_msg.content)
                    .footer(|f| f.text(&format!("{} | Zitiert von: {}", &quoted_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"), &msg.author.name)))
                )});
            }
        } else {
            let _ = msg.reply(&ctx.http, "Tut mir leid, ich kann diese Nachricht nicht finden.");
            trace!("Could not find quote message");
        }
    }
    let _ = msg.delete(&ctx.http);
    Ok(())
}