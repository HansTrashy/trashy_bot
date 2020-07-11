use crate::models::fav::Fav;
use crate::models::tag::Tag;
use crate::util::get_client;
use crate::DatabasePool;
use serenity::{model::channel::Reaction, prelude::*};
use std::time::Duration;
use tracing::trace;

pub async fn add(ctx: Context, add_reaction: Reaction) {
    let created_fav = Fav::create(
        &mut *get_client(&ctx).await.unwrap(),
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
    .expect("could not create fav");

    if let Ok(dm_channel) = add_reaction.user_id.unwrap().create_dm_channel(&ctx).await {
        trace!(user = ?add_reaction.user_id, "Requesting labels from user");

        let _ = dm_channel.say(&ctx, "Send me your labels!").await;

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
                let r =
                    Tag::create(&mut *get_client(&ctx).await.unwrap(), created_fav.id, tag).await;

                trace!(tag_creation = ?r, "Tag created");
            }

            let _ = label_reply.reply(&ctx, "added the tags!").await;
        }
    }
}
