use crate::models::fav::Fav;
use crate::models::fav_block::FavBlock;
use crate::models::tag::Tag;
use crate::util::get_client;
use serenity::{model::channel::Reaction, prelude::*};
use std::time::Duration;
use tracing::trace;

pub async fn add(ctx: Context, add_reaction: Reaction) {
    let pool = get_client(&ctx).await.unwrap();
    if FavBlock::check_blocked(
        &pool,
        *add_reaction.channel_id.as_u64() as i64,
        *add_reaction.message_id.as_u64() as i64,
    )
    .await
    {
        let channel = add_reaction
            .user_id
            .unwrap()
            .create_dm_channel(&ctx)
            .await
            .unwrap();

        let _ = channel.say(ctx, "This fav is blocked").await;
        return;
    }

    let created_fav = Fav::create(
        &pool,
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
        *add_reaction.user_id.unwrap().as_u64() as i64,
        *add_reaction
            .message(&ctx.http)
            .await
            .unwrap()
            .author
            .id
            .as_u64() as i64,
    )
    .await
    .expect("Could not create fav");

    if let Ok(dm_channel) = add_reaction.user_id.unwrap().create_dm_channel(&ctx).await {
        trace!(user = ?add_reaction.user_id, "Requesting tags from user");

        let content = format!(
            "Tags please! (space-separated): {}",
            add_reaction.message(&ctx.http).await.unwrap().content
        );
        let _ = dm_channel.say(&ctx, content).await;

        if let Some(label_reply) = dm_channel
            .id
            .await_reply(&ctx)
            .author_id(add_reaction.user_id.unwrap())
            .timeout(Duration::from_secs(120))
            .await
        {
            // fresh fav so it has no old tags to be deleted, just add the new ones

            // TODO: make this a single statement
            for tag in label_reply.content.split(' ') {
                let r = Tag::create(&pool, created_fav.id, tag).await;

                trace!(tag_creation = ?r, "Tag created!");
            }

            let _ = label_reply
                .reply(
                    &ctx,
                    "Tags added! To edit tags, react with 🏷️ on the posted fav",
                )
                .await;
        }
    }
}
