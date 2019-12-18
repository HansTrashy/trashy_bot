use crate::dispatch::DispatchEvent;
use crate::DispatcherKey;
use crate::OptOut;
use hey_listen::sync::ParallelDispatcherRequest as DispatcherRequest;
use lazy_static::lazy_static;
use log::*;
use regex::Regex;
use serenity::model::channel::Attachment;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::{Message, ReactionType},
};

#[command]
#[description = "Quote a message"]
#[usage = "command message-link"]
#[example = "https://discordapp.com/channels/_/_/_"]
#[only_in("guilds")]
pub fn quote(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let mut data = ctx.data.read();
    let opt_out = match data.get::<OptOut>() {
        Some(v) => v,
        None => {
            let _ = msg.reply(&ctx, "OptOut list not available");
            panic!("no optout");
        }
    };

    if opt_out.lock().set.contains(msg.author.id.as_u64()) {
        let _ = msg.channel_id.send_message(&ctx.http, |m| {
            m.content("You have opted out of the quote functionality")
        });
        let _ = msg.delete(&ctx);
        return Ok(());
    }

    lazy_static! {
        static ref QUOTE_LINK_REGEX: Regex =
            Regex::new(r#"https://discordapp.com/channels/(\d+)/(\d+)/(\d+)"#)
                .expect("could not compile quote link regex");
    }
    for caps in QUOTE_LINK_REGEX.captures_iter(&args.rest()) {
        let quote_server_id = caps[1].parse::<u64>()?;
        let quote_channel_id = caps[2].parse::<u64>()?;
        let quote_msg_id = caps[3].parse::<u64>()?;

        if let Ok(quoted_msg) = ChannelId(quote_channel_id).message(&ctx.http, quote_msg_id) {
            if opt_out.lock().set.contains(quoted_msg.author.id.as_u64()) {
                let _ = msg.channel_id.send_message(&ctx.http, |m| {
                    m.content("The user does not want to be quoted")
                });
                let _ = msg.delete(&ctx);
                return Ok(());
            }

            let bot_msg = msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    let mut embed = e
                        .author(|a| {
                            a.name(&quoted_msg.author.name).icon_url(
                                &quoted_msg.author.static_avatar_url().unwrap_or_default(),
                            )
                        })
                        .color((0, 120, 220))
                        .description(&quoted_msg.content)
                        .footer(|f| {
                            f.text(&format!(
                                "{} (UTC) | #{} | Quoted by: {}",
                                &quoted_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"),
                                &quoted_msg.channel_id.name(&ctx).unwrap_or("-".to_string()),
                                &msg.author.name
                            ))
                        });

                    if let Some(image) = quoted_msg
                        .attachments
                        .iter()
                        .cloned()
                        .filter(|a| a.width.is_some())
                        .collect::<Vec<Attachment>>()
                        .first()
                    {
                        embed = embed.image(&image.url);
                    }
                    embed
                })
            });
        } else {
            let _ = msg.reply(&ctx, "Sorry, i can not find this message.");
            trace!("Could not find quote message");
        }
    }

    let _ = msg.delete(&ctx);
    Ok(())
}
