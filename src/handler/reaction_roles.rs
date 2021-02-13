use crate::models::reaction_role::ReactionRole;
use crate::reaction_roles::State;
use crate::util::get_client;
use crate::ReactionRolesState;
use serenity::{
    model::{channel::Reaction, channel::ReactionType},
    prelude::*,
};
use tracing::{debug, error};

pub async fn add_role(ctx: Context, add_reaction: Reaction) {
    let rr_message_ids = match ctx.data.read().await.get::<ReactionRolesState>() {
        Some(v) => match *v.lock().await {
            State::Set(ref message_ids) => message_ids.clone(),
            State::NotSet => return,
        },
        None => return,
    };
    // check if reaction is on rr message
    if rr_message_ids.contains(add_reaction.message_id.as_u64()) {
        debug!("On correct message reacted!");
        if let ReactionType::Unicode(ref s) = add_reaction.emoji {
            // check if rr registered for this emoji
            let results = ReactionRole::list_by_emoji(&get_client(&ctx).await.unwrap(), s)
                .await
                .expect("Could not get by emojis");

            if !results.is_empty() {
                debug!("Found role for this emoji!");
                if let Some(guild) = add_reaction
                    .channel_id
                    .to_channel(&ctx)
                    .await
                    .ok()
                    .and_then(|c| c.guild())
                {
                    debug!("found guild for reaction");
                    if let Ok(mut member) = guild
                        .guild_id
                        .member(&ctx, add_reaction.user_id.unwrap())
                        .await
                    {
                        let result = member.add_role(&ctx, results[0].role_id as u64).await;
                        debug!(?result, "added role to user");
                    } else {
                        error!("failed to access member");
                    }
                } else {
                    error!("failed to access guild");
                }
            }
        }
    }
}

pub async fn remove_role(ctx: Context, remove_reaction: Reaction) {
    let rr_message_ids = match ctx.data.read().await.get::<ReactionRolesState>() {
        Some(v) => match *v.lock().await {
            State::Set(ref message_ids) => message_ids.clone(),
            State::NotSet => return,
        },
        None => return,
    };
    // check if reaction is on rr message
    if rr_message_ids.contains(remove_reaction.message_id.as_u64()) {
        debug!("On correct message reacted!");
        if let ReactionType::Unicode(ref s) = remove_reaction.emoji {
            // check if rr registered for this emoji
            let results = ReactionRole::list_by_emoji(&get_client(&ctx).await.unwrap(), s)
                .await
                .expect("Could not get by emojis");

            if !results.is_empty() {
                debug!("Found role for this emoji!");
                if let Some(guild) = remove_reaction
                    .channel_id
                    .to_channel(&ctx)
                    .await
                    .ok()
                    .and_then(|c| c.guild())
                {
                    debug!("found guild for reaction");
                    if let Ok(mut member) = guild
                        .guild_id
                        .member(&ctx, remove_reaction.user_id.unwrap())
                        .await
                    {
                        let result = member.remove_role(&ctx, results[0].role_id as u64).await;
                        debug!(?result, "added role to user");
                    } else {
                        error!("failed to access member");
                    }
                } else {
                    error!("failed to access guild");
                }
            }
        }
    }
}
