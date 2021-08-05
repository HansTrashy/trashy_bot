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
#[description = "Quote a message"]
#[usage = "*message_link*"]
#[example = "https://discord.com/channels/_/_/_"]
#[only_in("guilds")]
pub async fn quote(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    check_optout(ctx, msg, *msg.author.id.as_u64()).await?;

    let regex = crate::MESSAGE_REGEX.get().expect("Regex not initialized");

    let (quote_server_id, quote_channel_id, quote_msg_id) =
        util::parse_message_link(regex, args.rest())?;

    if let Ok(quoted_msg) = ChannelId(quote_channel_id)
        .message(&ctx.http, quote_msg_id)
        .await
    {
        check_optout(ctx, msg, *quoted_msg.author.id.as_u64()).await?;

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

        if msg.delete(ctx).await.is_err() {
            debug!("Deleting in dms is not supported");
        }

        let http = ctx.http.clone();
        bot_msg
            .await_reactions(&ctx)
            .timeout(Duration::from_secs(60 * 60_u64))
            .filter(|reaction| {
                matches!(reaction.emoji,
                ReactionType::Unicode(ref value) if value == "\u{2139}\u{fe0f}")
            })
            .await
            .for_each(|reaction| {
                let http = &http;
                async move {
                    // ignore add/remove reaction difference
                    let reaction = reaction.as_inner_ref();
                    if let Ok(dm_channel) = reaction.user_id.unwrap().create_dm_channel(http).await
                    {
                        trace!(user = ?reaction.user_id, "Sending info source for quote");
                        std::mem::drop(
                            dm_channel
                                .say(
                                    http,
                                    format!(
                                        "https://discord.com/channels/{}/{}/{}",
                                        quote_server_id, quote_channel_id, quote_msg_id,
                                    ),
                                )
                                .await,
                        );
                    }
                }
            })
            .await;
    } else {
        msg.reply(ctx, "Sorry, I can not find this message.")
            .await?;
        trace!("Could not find quote message");
    }

    Ok(())
}

async fn check_optout(ctx: &Context, msg: &Message, id: u64) -> CommandResult {
    if ctx
        .data
        .read()
        .await
        .get::<OptOut>()
        .expect("Expected optout")
        .lock()
        .await
        .set
        .contains(&id)
    {
        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.content("OptOut is used by you or the quoted")
            })
            .await?;
        msg.delete(ctx).await?;
        debug!("OptOut check unsuccessful");
        Err("Fav/Quote OptOut is active".into())
    } else {
        trace!("User id not contained in optout set");
        Ok(())
    }
}
