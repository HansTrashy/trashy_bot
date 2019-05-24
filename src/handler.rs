use crate::interaction::wait::Action;
use crate::models::tag::NewTag;
use crate::DatabaseConnection;
use crate::Waiter;
use diesel::prelude::*;
use log::*;
use serenity::{
    model::{channel::Message, channel::Reaction, channel::ReactionType, gateway::Ready},
    prelude::*,
};

mod blackjack;
mod fav;
mod reaction_roles;

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
                        Some(v) => v.get().unwrap(),
                        None => {
                            let _ = msg.reply("Could not retrieve the database connection!");
                            return;
                        }
                    };

                    // clear old tags for this fav
                    diesel::delete(tags.filter(fav_id.eq(waited_fav_id)))
                        .execute(&conn)
                        .expect("could not delete tags");

                    let received_tags: Vec<NewTag> = msg
                        .content
                        .split(' ')
                        .map(|t| NewTag::new(waited_fav_id, t.to_string()))
                        .collect();
                    crate::models::tag::create_tags(&conn, received_tags);

                    wait.purge(
                        *msg.author.id.as_u64(),
                        vec![Action::DeleteFav, Action::ReqTags, Action::AddTags],
                    );
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
            ReactionType::Unicode(ref s) if s == "👆" => blackjack::hit(ctx, add_reaction),
            ReactionType::Unicode(ref s) if s == "✋" => blackjack::stay(ctx, add_reaction),
            ReactionType::Unicode(ref s) if s == "🌀" => blackjack::new_game(ctx, add_reaction),
            _ => reaction_roles::add_role(ctx, add_reaction),
        }
    }

    fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        match removed_reaction.emoji {
            ReactionType::Unicode(ref s) if s == "👆" => blackjack::hit(ctx, removed_reaction),
            ReactionType::Unicode(ref s) if s == "✋" => blackjack::stay(ctx, removed_reaction),
            ReactionType::Unicode(ref s) if s == "🌀" => {
                blackjack::new_game(ctx, removed_reaction)
            }
            _ => reaction_roles::remove_role(ctx, removed_reaction),
        }
    }
}
