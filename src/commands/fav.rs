use crate::interaction::wait::{Action, WaitEvent};
use crate::models::favs::Fav;
use crate::models::tags::Tag;
use crate::schema::favs::dsl::*;
use crate::DatabaseConnection;
use crate::Waiter;
use chrono::prelude::*;
use diesel::prelude::*;
use rand::prelude::*;
use serenity::model::{channel::Message, channel::ReactionType, id::ChannelId, id::MessageId};

command!(fav(ctx, msg, args) {
    let mut rng = rand::thread_rng();
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };

    let labels: Vec<String> = args.iter::<String>().filter_map(Result::ok).collect();

    let results = favs.filter(user_id.eq(*msg.author.id.as_u64() as i64)).load::<Fav>(&*conn.lock()).expect("could not retrieve favs");

    let fav_tags = Tag::belonging_to(&results).load::<Tag>(&*conn.lock()).expect("could not retrieve tags").grouped_by(&results);
    let zipped = results.into_iter().zip(fav_tags).collect::<Vec<_>>();

    let possible_favs: Vec<(Fav, Vec<Tag>)> = zipped
            .into_iter()
            .filter_map(|(f, f_tags)| {
                for l in &labels {
                    let x = f_tags
                        .iter()
                        .fold(0, |acc, x| if &*x.label == l { acc + 1 } else { acc });
                    if x == 0 {
                        return None;
                    }
                }

                Some((f, f_tags))
            })
            .collect();

    let (chosen_fav, _tags) = possible_favs.iter().choose(&mut rng).unwrap();

    let fav_msg = ChannelId(chosen_fav.channel_id as u64).message(chosen_fav.msg_id as u64).unwrap();

    let _ = msg.delete();

    if let Some(waiter) = data.get::<Waiter>() {
        let mut wait = waiter.lock();

        //first remove all other waits for this user and these actions
        // dont do this until checked this is really necessary
        // => necessary for now, has to be changed wenn switching to async handling of this waiting thing
        wait.purge(*msg.author.id.as_u64(), vec![Action::DeleteFav, Action::ReqTags]);

        wait.wait(*msg.author.id.as_u64(), WaitEvent::new(Action::DeleteFav, chosen_fav.id, Utc::now()));
        wait.wait(*msg.author.id.as_u64(), WaitEvent::new(Action::ReqTags, chosen_fav.id, Utc::now()));
    }

    let _ = msg.channel_id.send_message(|m| m.embed(|e| 
        e.author(|a| a.name(&fav_msg.author.name).icon_url(&fav_msg.author.static_avatar_url().unwrap_or_default()))
        .description(&fav_msg.content)
        .color((0,120,220))
        .footer(|f| f.text(&format!("{} | Quoted by: {}", &fav_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"), &msg.author.name)))));
});

command!(untagged(ctx, msg, args) {
    if msg.is_private() {
        let data = ctx.data.lock();
        let conn = match data.get::<DatabaseConnection>() {
            Some(v) => v.clone(),
            None => {
                let _ = msg.reply("Could not retrieve the database connection!");
                return Ok(());
            }
        };

        let results = favs.filter(user_id.eq(*msg.author.id.as_u64() as i64)).load::<Fav>(&*conn.lock()).expect("could not retrieve favs");

        let fav_tags = Tag::belonging_to(&results).load::<Tag>(&*conn.lock()).expect("could not retrieve tags").grouped_by(&results);
        let zipped = results.into_iter().zip(fav_tags).collect::<Vec<_>>();

        let possible_favs: Vec<(Fav, Vec<Tag>)> = zipped
            .into_iter()
            .filter_map(|(f, f_tags)| {
                if f_tags.is_empty() {
                    Some((f, f_tags))
                } else {
                    None
                }
            })
            .collect();

        if !possible_favs.is_empty() {
            let (fa, _t) = possible_favs.first().unwrap();
            let fav_msg = ChannelId(fa.channel_id as u64).message(fa.msg_id as u64).unwrap();

            if let Some(waiter) = data.get::<Waiter>() {
                let mut wait = waiter.lock();

                wait.purge(*msg.author.id.as_u64(), vec![Action::DeleteFav, Action::ReqTags]);

                wait.wait(*msg.author.id.as_u64(), WaitEvent::new(Action::DeleteFav, fa.id, Utc::now()));
                wait.wait(*msg.author.id.as_u64(), WaitEvent::new(Action::ReqTags, fa.id, Utc::now()));
            }

            let sent_msg = msg.channel_id.send_message(|m| m.embed(|e| 
                e.author(|a| a.name(&fav_msg.author.name).icon_url(&fav_msg.author.static_avatar_url().unwrap_or_default()))
                .description(&fav_msg.content)
                .color((0,120,220))
                .footer(|f| f.text(&format!("{} | Quoted by: {}", &fav_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"), &msg.author.name)))));

            let sent_msg = sent_msg.unwrap();
            let _ = sent_msg.react(ReactionType::Unicode("🗑".to_string()));
            let _ = sent_msg.react(ReactionType::Unicode("🏷".to_string()));
        }

    } else {
        let _ = msg.channel_id.send_message(|m| m.content("Only works in DMs"));
    }
});

#[cfg(test)]
mod tests {
    use crate::models::favs::Fav;
    use crate::models::tags::Tag;
    use crate::schema::favs::dsl::*;

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
