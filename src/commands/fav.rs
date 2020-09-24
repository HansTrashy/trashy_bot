use crate::models::fav::Fav;
use crate::models::fav_block::FavBlock;
use crate::models::tag::Tag;
use crate::util;
use crate::util::get_client;
use crate::OptOut;
use futures::future::TryFutureExt;
use itertools::Itertools;
use lazy_static::lazy_static;
use rand::prelude::*;
use regex::Regex;
use serenity::futures::stream::StreamExt;
use serenity::model::{channel::Attachment, channel::ReactionType, id::ChannelId};
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    model::id::UserId,
};
use std::iter::FromIterator;
use std::time::Duration;
use tracing::{debug, trace};

#[command]
#[description = "Post a fav"]
#[example = "taishi wichsen"]
pub async fn post(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let opt_out = if let Some(v) = ctx.data.read().await.get::<OptOut>() {
        v.clone()
    } else {
        let _ = msg.reply(ctx, "OptOut list not available").await;
        panic!("no optout");
    };

    if opt_out.lock().await.set.contains(msg.author.id.as_u64()) {
        let _ = msg
            .channel_id
            .send_message(&ctx.http, |m| {
                m.content("You have opted out of the quote functionality")
            })
            .await;
        return Ok(());
    }

    let labels: Vec<String> = args.iter::<String>().filter_map(Result::ok).collect();

    let results = if labels.is_empty() {
        Fav::list(
            &mut *get_client(&ctx).await?,
            *msg.author.id.as_u64() as i64,
            msg.guild_id.map(|g_id| *g_id.as_u64() as i64),
        )
        .await?
    } else {
        Fav::tagged_with(
            &mut *get_client(&ctx).await?,
            *msg.author.id.as_u64() as i64,
            msg.guild_id.map(|g_id| *g_id.as_u64() as i64),
            labels,
        )
        .await?
    };

    let chosen_fav = results
        .into_iter()
        .choose(&mut rand::thread_rng())
        .ok_or("possible favs empty")?;

    let fav_msg = ChannelId(chosen_fav.channel_id as u64)
        .message(&ctx.http, chosen_fav.msg_id as u64)
        .await?;

    match msg.delete(ctx).await {
        Ok(_) => (),
        Err(_) => debug!("Deletion is not supported in DMs"),
    }

    if opt_out
        .lock()
        .await
        .set
        .contains(fav_msg.author.id.as_u64())
    {
        let _ = msg.channel_id.send_message(&ctx.http, |m| {
            m.content("The user does not want to be quoted")
        });
        return Ok(());
    }

    let channel_name = fav_msg
        .channel_id
        .name(&ctx)
        .await
        .unwrap_or_else(|| "-".to_string());

    let bot_msg = msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                let timestamp = fav_msg.timestamp.format("%d.%m.%Y, %H:%M:%S");

                let mut embed = e
                    .author(|a| {
                        a.name(&fav_msg.author.name)
                            .icon_url(&fav_msg.author.static_avatar_url().unwrap_or_default())
                    })
                    .description(&fav_msg.content)
                    .color((0, 120, 220))
                    .footer(|f| {
                        f.text(&format!(
                            "{} (UTC) | #{} | Fav by: {}",
                            timestamp, channel_name, &msg.author.name,
                        ))
                    });

                if let Some(image) = fav_msg
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

    let collector_delete = bot_msg
        .await_reactions(&ctx)
        .timeout(Duration::from_secs(120))
        .author_id(msg.author.id)
        .filter(|reaction| match reaction.emoji {
            ReactionType::Unicode(ref value) if value.starts_with("🗑") => true,
            _ => false,
        })
        .await;

    let collector_label = bot_msg
        .await_reactions(&ctx)
        .timeout(Duration::from_secs(120))
        .author_id(msg.author.id)
        .filter(|reaction| match reaction.emoji {
            ReactionType::Unicode(ref value) if value.starts_with("🏷") => true,
            _ => false,
        })
        .await;

    let collector_info = bot_msg
        .await_reactions(&ctx)
        .timeout(Duration::from_secs(5 * 60_u64))
        .filter(|reaction| match reaction.emoji {
            ReactionType::Unicode(ref value) if value == "ℹ\u{fe0f}" => true,
            _ => false,
        })
        .await;

    let c1 = collector_delete.for_each(|reaction| {
        let ctx = ctx.clone();
        let chosen_fav_id = chosen_fav.id;
        async move {
            let _ = Fav::delete(&mut *get_client(&ctx).await.unwrap(), chosen_fav_id).await;
        }
    });
    let c2 = collector_label.for_each(|reaction| {
        let ctx = ctx.clone();
        let chosen_fav_id = chosen_fav.id;
        async move {
            let reaction = reaction.as_inner_ref();
            if let Ok(dm_channel) = reaction.user_id.unwrap().create_dm_channel(&ctx).await {
                trace!(user = ?reaction.user_id, "Requesting labels from user");
                let _ = dm_channel.say(&ctx, "Send me your labels!").await;

                if let Some(label_reply) = dm_channel
                    .id
                    .await_reply(&ctx)
                    .author_id(reaction.user_id.unwrap())
                    .timeout(Duration::from_secs(120))
                    .await
                {
                    // clear old tags for this fav
                    let _ = Tag::delete(&mut *get_client(&ctx).await.unwrap(), chosen_fav_id).await;

                    // TODO: make this a single statement
                    for tag in label_reply.content.split(' ') {
                        let _ =
                            Tag::create(&mut *get_client(&ctx).await.unwrap(), chosen_fav_id, tag)
                                .await;
                    }

                    let _ = label_reply.reply(&ctx, "added the tags!").await;
                }
            }
        }
    });

    let c3 = collector_info.for_each(|reaction| {
        let ctx = ctx.clone();
        let chosen_fav = chosen_fav.clone();
        async move {
            // ignore add/remove reaction difference
            let reaction = reaction.as_inner_ref();
            if let Ok(dm_channel) = reaction.user_id.unwrap().create_dm_channel(&ctx).await {
                trace!(user = ?reaction.user_id, "sending info source for quote");
                let _ = dm_channel
                    .say(
                        &ctx,
                        format!(
                            "https://discordapp.com/channels/{}/{}/{}",
                            &chosen_fav.server_id, &chosen_fav.channel_id, &chosen_fav.msg_id,
                        ),
                    )
                    .await;
            }
        }
    });

    futures::future::join3(c1, c2, c3).await;

    Ok(())
}

#[command]
#[description = "Shows untagged favs so you can tag them"]
#[only_in("dms")]
#[num_args(0)]
pub async fn untagged(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let opt_out = match ctx.data.read().await.get::<OptOut>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply(ctx, "OptOut list not available");
            panic!("no optout");
        }
    };

    let results = Fav::untagged(
        &mut *get_client(&ctx).await.unwrap(),
        *msg.author.id.as_u64() as i64,
    )
    .await?;

    if results.is_empty() {
        let _ = msg.reply(ctx, "Du hat keine untagged Favs!").await;
    } else {
        let fav = results.first().unwrap();
        let fav_msg = ChannelId(fav.channel_id as u64)
            .message(&ctx, fav.msg_id as u64)
            .await
            .unwrap();

        if opt_out
            .lock()
            .await
            .set
            .contains(fav_msg.author.id.as_u64())
        {
            let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.content("The user does not want to be quoted")
            });
            return Ok(());
        }

        let bot_msg = msg
            .channel_id
            .send_message(&ctx, |m| {
                m.embed(|e| {
                    let mut embed = e
                        .author(|a| {
                            a.name(&fav_msg.author.name)
                                .icon_url(&fav_msg.author.static_avatar_url().unwrap_or_default())
                        })
                        .description(&fav_msg.content)
                        .color((0, 120, 220))
                        .footer(|f| {
                            f.text(&format!(
                                "{} | Zitiert von: {}",
                                &fav_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"),
                                &msg.author.name
                            ))
                        });

                    if let Some(image) = fav_msg
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

        let _ = bot_msg
            .react(ctx, ReactionType::Unicode("🗑".to_string()))
            .await;
        let _ = bot_msg
            .react(ctx, ReactionType::Unicode("🏷".to_string()))
            .await;

        let collector_delete = bot_msg
            .await_reactions(&ctx)
            .timeout(Duration::from_secs(120))
            .author_id(msg.author.id)
            .filter(|reaction| match reaction.emoji {
                ReactionType::Unicode(ref value) if value.starts_with("🗑") => true,
                _ => false,
            })
            .await;

        let collector_label = bot_msg
            .await_reactions(&ctx)
            .timeout(Duration::from_secs(120))
            .author_id(msg.author.id)
            .filter(|reaction| match reaction.emoji {
                ReactionType::Unicode(ref value) if value.starts_with("🏷") => true,
                _ => false,
            })
            .await;

        let c1 = collector_delete.for_each(|_| {
            let ctx = ctx.clone();
            let fav_id = fav.id;
            async move {
                trace!(fav = fav_id, "Delete Tag for fav");
                let _ = Fav::delete(&mut *get_client(&ctx).await.unwrap(), fav_id).await;
            }
        });
        let c2 = collector_label.for_each(|reaction| {
            let ctx = ctx.clone();
            let fav_id = fav.id;
            async move {
                let reaction = reaction.as_inner_ref();
                if let Ok(dm_channel) = reaction.user_id.unwrap().create_dm_channel(&ctx).await {
                    trace!(user = ?reaction.user_id, "Requesting labels from user");
                    let _ = dm_channel.say(&ctx, "Send me your labels!").await;

                    if let Some(label_reply) = dm_channel
                        .id
                        .await_reply(&ctx)
                        .author_id(reaction.user_id.unwrap())
                        .timeout(Duration::from_secs(120))
                        .await
                    {
                        // clear old tags for this fav
                        let r = Tag::delete(&mut *get_client(&ctx).await.unwrap(), fav_id).await;
                        trace!(tag_deletion = ?r, "Tags deleted");

                        // TODO: make this a single statement
                        for tag in label_reply.content.split(' ') {
                            let r = Tag::create(&mut *get_client(&ctx).await.unwrap(), fav_id, tag)
                                .await;

                            trace!(tag_creation = ?r, "Tag created");
                        }

                        let _ = label_reply.reply(&ctx, "added the tags!").await;
                    }
                }
            }
        });

        futures::future::join(c1, c2).await;
    }
    Ok(())
}

#[command]
#[only_in("dms")]
#[description = "Add a fav per link to the message"]
#[num_args(1)]
pub async fn add(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (fav_server_id, fav_channel_id, fav_msg_id) = util::parse_message_link(args.rest())?;

    let fav_msg = ChannelId(fav_channel_id)
        .message(&ctx.http, fav_msg_id)
        .await
        .expect("cannot find this message");

    if FavBlock::check_blocked(
        &mut *get_client(&ctx).await?,
        fav_channel_id as i64,
        fav_msg_id as i64,
    )
    .await
    {
        // is blocked
        msg.author
            .dm(ctx, |m| m.content("This fav is blocked"))
            .await?;
    } else {
        Fav::create(
            &mut *get_client(&ctx).await?,
            fav_server_id as i64,
            fav_channel_id as i64,
            fav_msg_id as i64,
            *msg.author.id.as_u64() as i64,
            *fav_msg.author.id.as_u64() as i64,
        )
        .await?;

        if let Err(why) = msg.author.dm(ctx, |m| m.content("Fav saved!")).await {
            debug!("Error sending message: {:?}", why);
        }
    }

    Ok(())
}

#[command]
#[only_in("dms")]
#[description = "Shows your used tags so you do not have to remember them all"]
#[num_args(0)]
pub async fn tags(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut messages = Vec::new();
    {
        let mut fav_tags = Tag::of_user(
            &mut *get_client(&ctx).await?,
            *msg.author.id.as_u64() as i64,
        )
        .await?;

        fav_tags.sort_unstable_by(|a, b| a.label.partial_cmp(&b.label).unwrap());
        let mut message_content = String::new();
        for (key, group) in &fav_tags.into_iter().group_by(|e| e.label.to_owned()) {
            message_content.push_str(&format!("{} ({})\n", key, group.count()));
        }

        for chunk in message_content.chars().chunks(1_500).into_iter() {
            messages.push(String::from_iter(chunk));
        }
    }

    let messages = messages
        .into_iter()
        .map(|description| {
            msg.channel_id
                .send_message(&ctx, |m| m.embed(|e| e.description(description)))
        })
        .collect::<Vec<_>>();

    futures::future::join_all(messages).await;

    Ok(())
}

#[command]
#[only_in("guilds")]
#[description = "Adds a fav to the blocklist"]
#[allowed_roles("Mods")]
pub async fn block(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (_, block_channel_id, block_msg_id) = util::parse_message_link(args.rest())?;

    // add to blocklist
    let fav_block = FavBlock::create(
        &mut *get_client(&ctx).await?,
        *msg.guild_id.unwrap().as_u64() as i64,
        block_channel_id as i64,
        block_msg_id as i64,
    )
    .await?;

    // check who used it as fav
    let favs_now_blocked = Fav::list_by_channel_msg(
        &mut *get_client(&ctx).await?,
        fav_block.channel_id,
        fav_block.msg_id,
    )
    .await?;

    // send messages to those using the fav
    futures::stream::iter(favs_now_blocked)
        .map(|blocked_fav| async move {
            if let Ok(dm_channel) = UserId(blocked_fav.user_id as u64)
                .create_dm_channel(&ctx)
                .await
            {
                let _ = dm_channel.say(ctx, format!("Es wurde gerade ein fav von dir geblockt. https://discordapp.com/channels/{}/{}/{}", blocked_fav.server_id as u64, blocked_fav.channel_id as u64, blocked_fav.msg_id as u64)).await;
            }
        })
        .collect::<Vec<_>>()
        .await;

    Ok(())
}

#[command]
#[only_in("guilds")]
#[description = "Creates a list of all favs on the server"]
#[allowed_roles("Mods")]
pub async fn create_fav_list(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    use tokio::io::{self, AsyncWriteExt, BufWriter};

    let favs = Fav::list_all_from_server(
        &mut *get_client(&ctx).await?,
        *msg.guild_id
            .ok_or("this command is only supposed to be called in a server channel")?
            .as_u64() as i64,
    )
    .await?;

    let outfile = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open("./fav_list.txt")
        .await?;

    let mut out_buf = BufWriter::new(outfile);

    for fav in favs {
        let fav_msg = ChannelId(fav.channel_id as u64)
            .message(&ctx, fav.msg_id as u64)
            .await?;

        let line = format!(
            "https://discord.com/channels/{}/{}/{} | {}\n",
            fav.server_id as u64, fav.channel_id as u64, fav.msg_id as u64, fav_msg.content
        );

        out_buf.write(line.as_bytes()).await?;
    }

    out_buf.flush().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::models::fav::Fav;
    use crate::models::tag::Tag;

    #[test]
    fn test_filter() {
        let input = vec![
            (
                Fav {
                    id: 1,
                    server_id: 1,
                    channel_id: 1,
                    msg_id: 1,
                    user_id: 1,
                    author_id: 1,
                },
                vec![
                    Tag {
                        id: 1,
                        fav_id: 1,
                        label: String::from("Haus"),
                    },
                    Tag {
                        id: 2,
                        fav_id: 1,
                        label: String::from("Fenster"),
                    },
                ],
            ),
            (
                Fav {
                    id: 2,
                    server_id: 2,
                    channel_id: 2,
                    msg_id: 2,
                    user_id: 2,
                    author_id: 2,
                },
                vec![
                    Tag {
                        id: 3,
                        fav_id: 2,
                        label: String::from("Auto"),
                    },
                    Tag {
                        id: 4,
                        fav_id: 2,
                        label: String::from("Haus"),
                    },
                ],
            ),
            (
                Fav {
                    id: 1,
                    server_id: 1,
                    channel_id: 1,
                    msg_id: 1,
                    user_id: 1,
                    author_id: 1,
                },
                vec![
                    Tag {
                        id: 1,
                        fav_id: 1,
                        label: String::from("Haus"),
                    },
                    Tag {
                        id: 2,
                        fav_id: 1,
                        label: String::from("Haus"),
                    },
                ],
            ),
            (
                Fav {
                    id: 1,
                    server_id: 1,
                    channel_id: 1,
                    msg_id: 1,
                    user_id: 1,
                    author_id: 1,
                },
                vec![
                    Tag {
                        id: 1,
                        fav_id: 1,
                        label: String::from("Haus"),
                    },
                    Tag {
                        id: 2,
                        fav_id: 1,
                        label: String::from("Turm"),
                    },
                ],
            ),
        ];

        let labels = vec!["Haus", "Turm", "Auto"];

        let possible_favs: Vec<(Fav, Vec<Tag>)> = input
            .into_iter()
            .filter_map(|(f, f_tags)| {
                for l in &labels {
                    let x = f_tags
                        .iter()
                        .fold(0, |acc, x| if &&*x.label == l { acc + 1 } else { acc });
                    if x == 0 {
                        return None;
                    }
                }

                Some((f, f_tags))
            })
            .collect();

        dbg!(&possible_favs);
    }
}
