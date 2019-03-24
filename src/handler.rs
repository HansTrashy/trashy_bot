use crate::interaction::wait::Action;
use crate::models::tag::NewTag;
use crate::DatabaseConnection;
use crate::Waiter;
use diesel::prelude::*;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use regex::Regex;
use serenity::{
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        help_commands, Args, CommandOptions, DispatchError, HelpBehaviour, StandardFramework,
    },
    model::{
        channel::Message, channel::Reaction, channel::ReactionType, gateway::Ready, id::ChannelId,
        Permissions,
    },
    prelude::*,
    utils::{content_safe, ContentSafeOptions},
};

// Regexes for bad words
lazy_static! {
    static ref BAD_WORDS: Vec<Regex> = { vec![Regex::new(r"ell[a|e]*").unwrap(),] };
}

mod fav;

pub struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    fn message(&self, ctx: Context, msg: Message) {
        info!("Message: {:?}", msg);
        if msg.is_private() {
            use crate::schema::tags::dsl::*;
            // check if waiting for labels
            let data = ctx.data.lock();
            if let Some(waiter) = data.get::<Waiter>() {
                let mut wait = waiter.lock();
                if let Some(waited_fav_id) = wait.waiting(*msg.author.id.as_u64(), Action::AddTags)
                {
                    let conn = match data.get::<DatabaseConnection>() {
                        Some(v) => v.clone(),
                        None => {
                            let _ = msg.reply("Could not retrieve the database connection!");
                            return;
                        }
                    };

                    // clear old tags for this fav
                    diesel::delete(tags.filter(fav_id.eq(waited_fav_id)))
                        .execute(&*conn.lock())
                        .expect("could not delete tags");

                    let received_tags: Vec<NewTag> = msg
                        .content
                        .split(' ')
                        .map(|t| NewTag::new(waited_fav_id, t.to_string()))
                        .collect();
                    crate::models::tag::create_tags(&*conn.lock(), received_tags);

                    let _ = msg.reply("added the tags!");
                }
            }
        } else if msg.author.id != 399343003233157124 && msg.channel_id != 385838671770943489 {
            info!("Bad word found!");
            // using wordfilter to check messages on guild for bad words
            let mut contains_bad_word = false;
            for r in BAD_WORDS.iter() {
                if r.is_match(&msg.content) {
                    contains_bad_word = true;
                }
            }
            let source = if msg.guild_id.is_some() {
                format!(
                    "https://discordapp.com/channels/{}/{}/{}",
                    msg.guild_id.unwrap(),
                    msg.channel_id,
                    msg.id,
                )
            } else {
                String::new()
            };
            if contains_bad_word {
                match ChannelId::from(559317647372713984).send_message(|m| {
                    m.embed(|e| {
                        e.author(|a| {
                            a.name(&msg.author.name)
                                .icon_url(&msg.author.static_avatar_url().unwrap_or_default())
                        })
                        .title(&format!(
                            "Potenzieller Verstoß in {}",
                            msg.channel_id.name().unwrap_or_default()
                        ))
                        .description(&msg.content)
                        .color((0, 120, 220))
                        .url(&source)
                        .footer(|f| {
                            f.text(&format!("{}", &msg.timestamp.format("%d.%m.%Y, %H:%M:%S"),))
                        })
                    })
                }) {
                    Ok(v) => (),
                    Err(e) => error!("Failure to send message: {}", e),
                }
            }
        }
    }

    fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        match add_reaction.emoji {
            ReactionType::Unicode(ref s) if s == "📗" => fav::add_fav(ctx, add_reaction),
            ReactionType::Unicode(ref s) if s == "🗑" => fav::remove_fav(ctx, add_reaction),
            ReactionType::Unicode(ref s) if s == "🏷" => fav::add_label(ctx, add_reaction),
            _ => (),
        }
    }
}
