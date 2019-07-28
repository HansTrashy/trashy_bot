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
use crate::DispatcherKey;
use crate::dispatch::DispatchEvent;
use hey_listen::sync::ParallelDispatcherRequest as DispatcherRequest;

#[command]
#[description = "Quote a message"]
#[usage = "command message-link"]
#[example = "https://discordapp.com/channels/_/_/_"]
#[only_in("guilds")]
pub fn quote(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    lazy_static! {
        static ref QUOTE_LINK_REGEX: Regex =
            Regex::new(r#"https://discordapp.com/channels/\d+/(\d+)/(\d+)"#)
                .expect("could not compile quote link regex");
    }
    for caps in QUOTE_LINK_REGEX.captures_iter(&args.rest()) {
        let quote_channel_id = caps[1].parse::<u64>()?;
        let quote_msg_id = caps[2].parse::<u64>()?;

        let dispatcher = {
            let mut context = ctx.data.write();
            context.get_mut::<DispatcherKey>().expect("Expected Dispatcher.").clone()
        };

        if let Ok(quoted_msg) = ChannelId(quote_channel_id).message(&ctx.http, quote_msg_id) {
            if let Some(image) = quoted_msg
                .attachments
                .iter()
                .cloned()
                .filter(|a| a.width.is_some())
                .collect::<Vec<Attachment>>()
                .first()
            {
                let bot_msg = msg.channel_id.send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.author(|a| {
                            a.name(&quoted_msg.author.name).icon_url(
                                &quoted_msg.author.static_avatar_url().unwrap_or_default(),
                            )
                        })
                        .color((0, 120, 220))
                        .description(&quoted_msg.content)
                        .image(&image.url)
                        .footer(|f| {
                            f.text(&format!(
                                "{} | Zitiert von: {}",
                                &quoted_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"),
                                &msg.author.name
                            ))
                        })
                    })
                });


                let http = ctx.http.clone();
                if let Ok(bot_msg) = bot_msg {
                    dispatcher.write()
                    .add_fn(DispatchEvent::ReactEvent(bot_msg.id, msg.author.id),
                        Box::new(move |event: &DispatchEvent| {
                            if let DispatchEvent::ReactEvent(msg_id, author_id) = &event {
                                if let Ok(dm_channel) = author_id.create_dm_channel(&http) {
                                    dm_channel.say(&http, "");
                                }
                            }
                            Some(DispatcherRequest::StopListening)
                        }));
                }
            } else {
                let _ = msg.channel_id.send_message(&ctx, |m| {
                    m.embed(|e| {
                        e.author(|a| {
                            a.name(&quoted_msg.author.name).icon_url(
                                &quoted_msg.author.static_avatar_url().unwrap_or_default(),
                            )
                        })
                        .color((0, 120, 220))
                        .description(&quoted_msg.content)
                        .footer(|f| {
                            f.text(&format!(
                                "{} | Zitiert von: {}",
                                &quoted_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"),
                                &msg.author.name
                            ))
                        })
                    })
                });
            }
        } else {
            let _ = msg.reply(&ctx, "Tut mir leid, ich kann diese Nachricht nicht finden.");
            trace!("Could not find quote message");
        }
    }
    let _ = msg.delete(&ctx);
    Ok(())
}
