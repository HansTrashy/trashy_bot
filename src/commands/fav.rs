use crate::dispatch::{DispatchEvent, Listener};
use crate::interaction::wait::{Action, Event};
use crate::models::fav::Fav;
use crate::models::tag::Tag;
use crate::DatabaseConnection;
use crate::OptOut;
use crate::Waiter;
use chrono::prelude::*;
use itertools::Itertools;
use lazy_static::lazy_static;
use log::*;
use rand::prelude::*;
use regex::Regex;
use serenity::model::{channel::Attachment, channel::ReactionType, id::ChannelId};
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use std::iter::FromIterator;

#[command]
#[description = "Post a fav"]
#[example = "taishi wichsen"]
pub fn post(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut rng = rand::thread_rng();
    let mut data = ctx.data.write();
    let mut conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().expect("could not get conn from pool"),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let dispatcher = {
        data.get_mut::<crate::TrashyDispatcher>()
            .expect("Expected Dispatcher.")
            .clone()
    };
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
        return Ok(());
    }

    let labels: Vec<String> = args.iter::<String>().filter_map(Result::ok).collect();

    let results = Fav::tagged_with(&mut *conn, *msg.author.id.as_u64() as i64, labels)?;

    let chosen_fav = results
        .into_iter()
        .choose(&mut rng)
        .expect("possible favs are empty");

    let fav_msg = ChannelId(chosen_fav.channel_id as u64)
        .message(&ctx.http, chosen_fav.msg_id as u64)
        .expect("no fav message exists for this id");

    let _ = msg.delete(&ctx);

    if opt_out.lock().set.contains(fav_msg.author.id.as_u64()) {
        let _ = msg.channel_id.send_message(&ctx.http, |m| {
            m.content("The user does not want to be quoted")
        });
        return Ok(());
    }

    if let Some(waiter) = data.get::<Waiter>() {
        let mut wait = waiter.lock();

        //first remove all other waits for this user and these actions
        // dont do this until checked this is really necessary
        // => necessary for now, has to be changed wenn switching to async handling of this waiting thing
        wait.purge(
            *msg.author.id.as_u64(),
            vec![Action::DeleteFav, Action::ReqTags],
        );

        wait.wait(
            *msg.author.id.as_u64(),
            Event::new(Action::DeleteFav, chosen_fav.id, Utc::now()),
        );
        wait.wait(
            *msg.author.id.as_u64(),
            Event::new(Action::ReqTags, chosen_fav.id, Utc::now()),
        );
    }

    let bot_msg = msg.channel_id.send_message(&ctx.http, |m| {
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
                        "{} (UTC) | #{} | Fav by: {}",
                        &fav_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"),
                        &fav_msg
                            .channel_id
                            .name(&ctx)
                            .unwrap_or_else(|| "-".to_string()),
                        &msg.author.name,
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
    });

    let http = ctx.http.clone();
    if let Ok(bot_msg) = bot_msg {
        dispatcher.lock().add_listener(
            DispatchEvent::ReactMsg(
                bot_msg.id,
                ReactionType::Unicode("ℹ️".to_string()),
                bot_msg.channel_id,
                msg.author.id,
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
                                    chosen_fav.server_id, chosen_fav.channel_id, chosen_fav.msg_id,
                                ),
                            );
                        }
                    }
                }),
            ),
        );
    }
    Ok(())
}

#[command]
#[description = "Shows untagged favs so you can tag them"]
#[only_in("dms")]
#[num_args(0)]
pub fn untagged(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read();
    let mut conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let opt_out = match data.get::<OptOut>() {
        Some(v) => v,
        None => {
            let _ = msg.reply(&ctx, "OptOut list not available");
            panic!("no optout");
        }
    };

    let results = Fav::untagged(&mut *conn, *msg.author.id.as_u64() as i64)?;

    if results.is_empty() {
        let _ = msg.reply(&ctx, "Du hat keine untagged Favs!");
    } else {
        let fa = results.first().unwrap();
        let fav_msg = ChannelId(fa.channel_id as u64)
            .message(&ctx, fa.msg_id as u64)
            .unwrap();

        if opt_out.lock().set.contains(fav_msg.author.id.as_u64()) {
            let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.content("The user does not want to be quoted")
            });
            return Ok(());
        }

        if let Some(waiter) = data.get::<Waiter>() {
            let mut wait = waiter.lock();

            wait.purge(
                *msg.author.id.as_u64(),
                vec![Action::DeleteFav, Action::ReqTags],
            );

            wait.wait(
                *msg.author.id.as_u64(),
                Event::new(Action::DeleteFav, fa.id, Utc::now()),
            );
            wait.wait(
                *msg.author.id.as_u64(),
                Event::new(Action::ReqTags, fa.id, Utc::now()),
            );
        }

        let sent_msg = msg.channel_id.send_message(&ctx, |m| {
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
        });

        let sent_msg = sent_msg.unwrap();
        let _ = sent_msg.react(&ctx, ReactionType::Unicode("🗑".to_string()));
        let _ = sent_msg.react(&ctx, ReactionType::Unicode("🏷".to_string()));
    }
    Ok(())
}

#[command]
#[only_in("dms")]
#[description = "Add a fav per link to the message"]
#[num_args(1)]
pub fn add(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read();
    lazy_static! {
        static ref FAV_LINK_REGEX: Regex =
            Regex::new(r#"https://discordapp.com/channels/(\d+)/(\d+)/(\d+)"#)
                .expect("couldnt compile quote link regex");
    }
    for caps in FAV_LINK_REGEX.captures_iter(&args.rest()) {
        let fav_server_id = caps[1].parse::<u64>().unwrap();
        let fav_channel_id = caps[2].parse::<u64>().unwrap();
        let fav_msg_id = caps[3].parse::<u64>().unwrap();

        let fav_msg = ChannelId(fav_channel_id)
            .message(&ctx.http, fav_msg_id)
            .expect("cannot find this message");

        if let Some(pool) = data.get::<DatabaseConnection>() {
            let mut conn = pool.get().unwrap();
            Fav::create(
                &mut *conn,
                fav_server_id as i64,
                fav_channel_id as i64,
                fav_msg_id as i64,
                *msg.author.id.as_u64() as i64,
                *fav_msg.author.id.as_u64() as i64,
            )?;

            if let Err(why) = msg.author.dm(&ctx, |m| m.content("Fav saved!")) {
                debug!("Error sending message: {:?}", why);
            }
        }
    }
    Ok(())
}

#[command]
#[only_in("dms")]
#[description = "Shows your used tags so you do not have to remember them all"]
#[num_args(0)]
pub fn tags(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read();
    let mut conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };

    let mut fav_tags = Tag::of_user(&mut *conn, *msg.author.id.as_u64() as i64)?;

    fav_tags.sort_unstable_by(|a, b| a.label.partial_cmp(&b.label).unwrap());

    let mut message_content = String::new();
    for (key, group) in &fav_tags.into_iter().group_by(|e| e.label.to_owned()) {
        message_content.push_str(&format!("{} ({})\n", key, group.count()));
    }

    message_content
        .chars()
        .chunks(1_500)
        .into_iter()
        .for_each(|chunk| {
            let _ = msg.channel_id.send_message(&ctx, |m| {
                m.embed(|e| e.description(&String::from_iter(chunk)))
            });
        });

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
