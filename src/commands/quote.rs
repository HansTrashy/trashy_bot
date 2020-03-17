use crate::dispatch::{Event as DispatchEvent, Listener};
use crate::OptOut;
use crate::TrashyDispatcher;
use lazy_static::lazy_static;
use regex::Regex;
use serenity::model::channel::Attachment;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::{Message, ReactionType},
};
use tracing::{info, trace};

#[command]
#[description = "Quote a message"]
#[usage = "command message-link"]
#[example = "https://discordapp.com/channels/_/_/_"]
#[only_in("guilds")]
pub async fn quote(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.write().await;

    if data
        .get::<OptOut>()
        .expect("expected optout")
        .lock()
        .await
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
    // for caps in QUOTE_LINK_REGEX.captures_iter(args.rest()) {
    let caps = QUOTE_LINK_REGEX
        .captures(args.rest())
        .ok_or("No captures, invalid link?")?;
    let quote_server_id = caps.get(1).map_or("", |m| m.as_str()).parse::<u64>()?;
    let quote_channel_id = caps.get(2).map_or("", |m| m.as_str()).parse::<u64>()?;
    let quote_msg_id = caps.get(3).map_or("", |m| m.as_str()).parse::<u64>()?;

    if let Ok(quoted_msg) = ChannelId(quote_channel_id)
        .message(&ctx.http, quote_msg_id)
        .await
    {
        if data
            .get::<OptOut>()
            .expect("expected optout")
            .lock()
            .await
            .set
            .contains(quoted_msg.author.id.as_u64())
        {
            let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.content("The user does not want to be quoted")
            });
            let _ = msg.delete(&ctx);
            return Ok(());
        }

        let channel_name = quoted_msg
            .channel_id
            .name(&ctx)
            .await
            .unwrap_or_else(|| "-".to_string());
        let bot_msg = msg
            .channel_id
            .send_message(&ctx.http, |m| {
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
                                channel_name,
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
            })
            .await?;

        let http = ctx.http.clone();
        if let Some(dispatcher) = data.get::<TrashyDispatcher>() {
            dispatcher.lock().await.add_listener(
                DispatchEvent::ReactMsg(
                    bot_msg.id,
                    ReactionType::Unicode("ℹ️".to_string()),
                    bot_msg.channel_id,
                    bot_msg.author.id,
                ),
                Listener::new(
                    std::time::Duration::from_secs(60 * 60),
                    Box::new(move |_, event| {
                        info!(event = ?event, "executing futures reaction");
                        let http = http.clone();
                        Box::pin(async move {
                            if let DispatchEvent::ReactMsg(
                                _msg_id,
                                _reaction_type,
                                _channel_id,
                                react_user_id,
                            ) = &event
                            {
                                if let Ok(dm_channel) =
                                    react_user_id.create_dm_channel(&http.clone()).await
                                {
                                    let _ = dm_channel
                                        .say(
                                            &http,
                                            format!(
                                                "https://discordapp.com/channels/{}/{}/{}",
                                                quote_server_id, quote_channel_id, quote_msg_id,
                                            ),
                                        )
                                        .await;
                                }
                            }
                        })
                    }),
                ),
            );
        }
    } else {
        let _ = msg.reply(&ctx, "Sorry, i can not find this message.").await;
        trace!("Could not find quote message");
    }

    let _ = msg.delete(&ctx);
    Ok(())
}
