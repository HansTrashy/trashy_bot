use crate::interaction::wait::Action;
use crate::models::tags::NewTag;
use crate::DatabaseConnection;
use crate::Waiter;
use diesel::prelude::*;
use serenity::{
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        help_commands, Args, CommandOptions, DispatchError, HelpBehaviour, StandardFramework,
    },
    model::{
        channel::Message, channel::Reaction, channel::ReactionType, gateway::Ready, Permissions,
    },
    prelude::*,
    utils::{content_safe, ContentSafeOptions},
};

mod fav;

pub struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    fn message(&self, ctx: Context, msg: Message) {
        use crate::schema::tags::dsl::*;
        if msg.is_private() {
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
                    crate::models::tags::create_tags(&*conn.lock(), received_tags);

                    let _ = msg.reply("added the tags!");
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
