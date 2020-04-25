use crate::interaction::wait::Action;
use crate::interaction::wait::Event;
use crate::models::fav::Fav;
use crate::DatabasePool;
use crate::Waiter;
use chrono::prelude::*;
use serenity::{model::channel::Reaction, prelude::*};

pub async fn add(ctx: Context, add_reaction: Reaction) {

    if let Some(pool) = ctx.data.read().await.get::<DatabasePool>() {
        let mut conn = pool.get().await.unwrap();
        let created_fav = Fav::create(
            &mut *conn,
            *add_reaction
                .channel(&ctx)
                .await
                .unwrap()
                .guild()
                .unwrap()
                .guild_id
                .as_u64() as i64,
            *add_reaction.channel_id.as_u64() as i64,
            *add_reaction.message_id.as_u64() as i64,
            *add_reaction.user_id.as_u64() as i64,
            *add_reaction
                .message(&ctx.http)
                .await
                .unwrap()
                .author
                .id
                .as_u64() as i64,
        )
        .await
        .expect("could not create fav");

        if let Some(waiter) = ctx.data.read().await.get::<Waiter>() {
            let mut wait = waiter.lock().await;
            wait.wait(
                *add_reaction.user_id.as_u64(),
                Event::new(Action::AddTags, created_fav.id, Utc::now()),
            );
        }

        if let Err(why) = add_reaction
            .user(&ctx)
            .await
            .unwrap()
            .dm(&ctx, |m| m.content("Send me your labels!"))
            .await
        {
            println!("Error sending message: {:?}", why);
        }
    }
}

pub async fn add_label(ctx: Context, add_reaction: Reaction) {
    if let Some(waiter) = ctx.data.read().await.get::<Waiter>() {
        let mut wait = waiter.lock().await;
        if let Some(fav_id) = wait.waiting(*add_reaction.user_id.as_u64(), &Action::ReqTags) {
            wait.wait(
                *add_reaction.user_id.as_u64(),
                Event::new(Action::AddTags, fav_id, Utc::now()),
            );
        }

        // send message for labels
        let _ = add_reaction
            .user(&ctx)
            .await
            .unwrap()
            .dm(&ctx, |m| m.content("Please send me your tags."))
            .await;
    }
}

pub async fn remove(ctx: Context, add_reaction: Reaction) {
    // check if waiting for deletion
    if let Some(waiter) = ctx.data.read().await.get::<Waiter>() {
        let mut wait = waiter.lock().await;
        if let Some(fav_id) = wait.waiting(*add_reaction.user_id.as_u64(), &Action::DeleteFav) {
            if let Some(pool) = ctx.data.read().await.get::<DatabasePool>() {
                let mut conn = pool.get().await.unwrap();
                Fav::delete(&mut *conn, fav_id)
                    .await
                    .expect("could not delete fav");
            }
        }
    }
}
