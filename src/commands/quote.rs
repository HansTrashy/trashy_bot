use lazy_static::lazy_static;
use log::*;
use regex::Regex;
use serenity::model::channel::Attachment;
use serenity::model::id::ChannelId;

command!(quote(_ctx, msg, args) {
    lazy_static! {
        static ref QUOTE_LINK_REGEX: Regex = Regex::new(r#"https://discordapp.com/channels/\d+/(\d+)/(\d+)"#)
            .expect("couldnt compile quote link regex");
    }
    for caps in QUOTE_LINK_REGEX.captures_iter(&args.rest()) {
        let quote_channel_id = caps[1].parse::<u64>().unwrap();
        let quote_msg_id = caps[2].parse::<u64>().unwrap();

        if let Ok(quoted_msg) = ChannelId(quote_channel_id).message(quote_msg_id) {
            if let Some(image) = quoted_msg.attachments.iter().cloned().filter(|a| a.width.is_some()).collect::<Vec<Attachment>>().first() {
                let _ = msg.channel_id.send_message(|m| {
                m.embed(|e|
                    e.author(|a| a.name(&quoted_msg.author.name).icon_url(&quoted_msg.author.static_avatar_url().unwrap_or_default()))
                    .color((0,120,220))
                    .description(&quoted_msg.content)
                    .image(&image.url)
                    .footer(|f| f.text(&format!("{} | Zitiert von: {}", &quoted_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"), &msg.author.name)))
                )});
            } else {
                let _ = msg.channel_id.send_message(|m| {
                m.embed(|e|
                    e.author(|a| a.name(&quoted_msg.author.name).icon_url(&quoted_msg.author.static_avatar_url().unwrap_or_default()))
                    .color((0,120,220))
                    .description(&quoted_msg.content)
                    .footer(|f| f.text(&format!("{} | Zitiert von: {}", &quoted_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"), &msg.author.name)))
                )});
            }
        } else {
            let _ = msg.reply("Tut mir leid, ich kann diese Nachricht nicht finden.");
            trace!("Could not find quote message");
        }
    }
    let _ = msg.delete();
});
