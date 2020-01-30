use crate::dispatch::{DispatchEvent, Listener};
use crate::OptOut;
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
    let mut data = ctx.data.write();

    if data
        .get::<OptOut>()
        .expect("expected optout")
        .lock()
        .set
        .contains(msg.author.id.as_u64())
    {
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
            if data
                .get::<OptOut>()
                .expect("expected optout")
                .lock()
                .set
                .contains(quoted_msg.author.id.as_u64())
            {
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
                                &quoted_msg
                                    .channel_id
                                    .name(&ctx)
                                    .unwrap_or_else(|| "-".to_string()),
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
            })?;

            let dispatcher = {
                data.get_mut::<crate::TrashyDispatcher>()
                    .expect("Expected Dispatcher.")
                    .clone()
            };

            let http = ctx.http.clone();

            dispatcher.lock().add_listener(
                DispatchEvent::ReactMsg(
                    bot_msg.id,
                    ReactionType::Unicode("ℹ️".to_string()),
                    bot_msg.channel_id,
                    bot_msg.author.id,
                ),
                Listener::new(
                    std::time::Duration::from_secs(60 * 60),
                    Box::new(move |_, event| {
                        if let DispatchEvent::ReactMsg(
                            _msg_id,
                            _reaction_type,
                            _channel_id,
                            react_user_id,
                        ) = &event
                        {
                            if let Ok(dm_channel) = react_user_id.create_dm_channel(&http) {
                                let _ = dm_channel.say(
                                    &http,
                                    format!(
                                        "https://discordapp.com/channels/{}/{}/{}",
                                        quote_server_id, quote_channel_id, quote_msg_id,
                                    ),
                                );
                            }
                        }
                    }),
                ),
            );
        } else {
            let _ = msg.reply(&ctx, "Sorry, i can not find this message.");
            trace!("Could not find quote message");
        }
    }

    let _ = msg.delete(&ctx);
    Ok(())
}
