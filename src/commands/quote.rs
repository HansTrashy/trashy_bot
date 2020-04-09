use crate::OptOut;
use lazy_static::lazy_static;
use regex::Regex;
use serenity::collector::reaction_collector::ReactionCollectorBuilder;
use serenity::model::channel::Attachment;
use serenity::model::id::ChannelId;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
};
use std::time::Duration;
use tracing::{debug, trace};

lazy_static! {
    static ref QUOTE_LINK_REGEX: Regex =
        Regex::new(r#"https://discordapp.com/channels/(\d+)/(\d+)/(\d+)"#)
            .expect("could not compile quote link regex");
}

#[command]
#[description = "Quote a message"]
#[usage = "command message-link"]
#[example = "https://discordapp.com/channels/_/_/_"]
#[only_in("guilds")]
pub async fn quote(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    check_optout(ctx, msg, msg.author.id.as_u64()).await?;

    let caps = QUOTE_LINK_REGEX
        .captures(args.rest())
        .ok_or("No captures, invalid link?")?;
    let _quote_server_id = caps.get(1).map_or("", |m| m.as_str()).parse::<u64>()?;
    let quote_channel_id = caps.get(2).map_or("", |m| m.as_str()).parse::<u64>()?;
    let quote_msg_id = caps.get(3).map_or("", |m| m.as_str()).parse::<u64>()?;

    if let Ok(quoted_msg) = ChannelId(quote_channel_id)
        .message(&ctx.http, quote_msg_id)
        .await
    {
        check_optout(ctx, msg, quoted_msg.author.id.as_u64()).await?;

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

        match msg.delete(&ctx).await {
            Ok(_) => (),
            Err(_) => debug!("deleting in dms is not supported"),
        }

    //TODO: implement this when the collector support for this is done

    // let mut collector = ReactionCollectorBuilder::new(&ctx)
    //     .message_id(bot_msg.id)
    //     .timeout(Duration::from_secs(5))
    //     .await;

    // let http = ctx.http.clone();
    // let _ = tokio::time::timeout(Duration::from_secs(60 * 60_u64), async move {
    //     loop {
    //         if let Some(reaction) = collector.receive_one().await {
    //             if let Ok(dm_channel) = reaction
    //                 .as_inner_ref()
    //                 .user_id
    //                 .create_dm_channel(&http.clone())
    //                 .await
    //             {
    //                 trace!(user = ?reaction.as_inner_ref().user_id, "sending user info source for quote");
    //                 let _ = dm_channel
    //                     .say(
    //                         &http.clone(),
    //                         format!(
    //                             "https://discordapp.com/channels/{}/{}/{}",
    //                             quote_server_id, quote_channel_id, quote_msg_id,
    //                         ),
    //                     )
    //                     .await;
    //             }
    //         }
    //     }
    // })
    // .await;
    } else {
        msg.reply(&ctx, "Sorry, i can not find this message.")
            .await?;
        trace!("Could not find quote message");
    }

    Ok(())
}

async fn check_optout(ctx: &mut Context, msg: &Message, id: &u64) -> CommandResult {
    if ctx
        .data
        .read()
        .await
        .get::<OptOut>()
        .expect("expected optout")
        .lock()
        .await
        .set
        .contains(id)
    {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.content("OptOut is used by you or the quoted")
            })
            .await?;
        msg.delete(&ctx).await?;
        Ok(())
    } else {
        debug!("OptOut check unsuccessful");
        Err("Fav/Quote OptOut is active".into())
    }
}
