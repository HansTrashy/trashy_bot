use crate::interaction::wait::Action;
use crate::interaction::wait::WaitEvent;
use crate::DatabaseConnection;
use crate::Waiter;
use chrono::prelude::*;
use diesel::prelude::*;
use serenity::{model::channel::Reaction, prelude::*};

pub fn add_fav(ctx: Context, add_reaction: Reaction) {
    let data = ctx.data.read();

    if let Some(pool) = data.get::<DatabaseConnection>() {
        let conn: &PgConnection = &pool.get().unwrap();
        let created_fav = crate::models::fav::create_fav(
            conn,
            *add_reaction
                .channel(&ctx)
                .unwrap()
                .guild()
                .unwrap()
                .read()
                .guild_id
                .as_u64() as i64,
            *add_reaction.channel_id.as_u64() as i64,
            *add_reaction.message_id.as_u64() as i64,
            *add_reaction.user_id.as_u64() as i64,
            *add_reaction.message(&ctx.http).unwrap().author.id.as_u64() as i64,
        );

        if let Some(waiter) = data.get::<Waiter>() {
            let mut wait = waiter.lock();
            wait.wait(
                *add_reaction.user_id.as_u64(),
                WaitEvent::new(Action::AddTags, created_fav.id, Utc::now()),
            );
        }

        if let Err(why) = add_reaction
            .user(&ctx)
            .unwrap()
            .dm(&ctx, |m| m.content("Send me your labels!"))
        {
            println!("Error sending message: {:?}", why);
        }
    }
}

pub fn add_label(ctx: Context, add_reaction: Reaction) {
    let data = ctx.data.read();

    if let Some(waiter) = data.get::<Waiter>() {
        let mut wait = waiter.lock();
        if let Some(fav_id) = wait.waiting(*add_reaction.user_id.as_u64(), Action::ReqTags) {
            wait.wait(
                *add_reaction.user_id.as_u64(),
                WaitEvent::new(Action::AddTags, fav_id, Utc::now()),
            );
        }

        // send message for labels
        let _ = add_reaction
            .user(&ctx)
            .unwrap()
            .dm(&ctx, |m| m.content("Please send me your tags."));
    }
}

pub fn remove_fav(ctx: Context, add_reaction: Reaction) {
    use crate::schema::favs::dsl::*;
    let data = ctx.data.read();

    // check if waiting for deletion
    if let Some(waiter) = data.get::<Waiter>() {
        let mut wait = waiter.lock();
        if let Some(fav_id) = wait.waiting(*add_reaction.user_id.as_u64(), Action::DeleteFav) {
            if let Some(pool) = data.get::<DatabaseConnection>() {
                let conn: &PgConnection = &pool.get().unwrap();
                diesel::delete(favs.filter(id.eq(fav_id)))
                    .execute(conn)
                    .expect("could not delete fav");
            }
        }
    }
}
