use crate::interaction::wait::Action;
use crate::DatabaseConnection;
use crate::Waiter;
use diesel::pg::PgConnection;
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

pub fn add_fav(ctx: Context, add_reaction: Reaction) {
    let data = ctx.data.lock();

    if let Some(conn) = data.get::<DatabaseConnection>() {
        crate::models::favs::create_fav(
            &*conn.lock(),
            *add_reaction
                .channel()
                .unwrap()
                .guild()
                .unwrap()
                .read()
                .guild_id
                .as_u64() as i64,
            *add_reaction.channel_id.as_u64() as i64,
            *add_reaction.message_id.as_u64() as i64,
            *add_reaction.user_id.as_u64() as i64,
            *add_reaction.message().unwrap().author.id.as_u64() as i64,
        );

        if let Err(why) = add_reaction.user().unwrap().dm(|m| m.content("Fav saved!")) {
            println!("Error sending message: {:?}", why);
        }
    }
}

pub fn remove_fav(ctx: Context, add_reaction: Reaction) {
    use crate::schema::favs::dsl::*;
    let data = ctx.data.lock();
    // println!("trying to remove fav");

    // check if waiting for deletion
    if let Some(waiter) = data.get::<Waiter>() {
        let mut wait = waiter.lock();
        if let Some(fav_id) = wait.waiting(*add_reaction.user_id.as_u64(), Action::DeleteFav) {
            if let Some(conn) = data.get::<DatabaseConnection>() {
                diesel::delete(favs.filter(id.eq(fav_id)))
                    .execute(&*conn.lock())
                    .expect("could not delete fav");
                // println!("Deleted fav");
            }
        }
    }
}
