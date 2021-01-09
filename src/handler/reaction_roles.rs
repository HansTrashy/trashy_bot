use crate::models::reaction_role::ReactionRole;
use crate::reaction_roles::State;
use crate::util::get_client;
use crate::ReactionRolesState;
use serenity::{
    model::{channel::Reaction, channel::ReactionType},
    prelude::*,
};
use tracing::info;

pub async fn add_role(ctx: Context, reaction: Reaction) {
    handle_reaction(ctx, reaction, true).await;
}

pub async fn remove_role(ctx: Context, reaction: Reaction) {
    handle_reaction(ctx, reaction, false).await;
}

async fn handle_reaction(ctx: Context, reaction: Reaction, reaction_added: bool) {
    let (rr_channel_id, rr_message_ids) = match ctx.data.read().await.get::<ReactionRolesState>() {
        Some(v) => match *v.lock().await {
            State::Set {
                ref channel_id,
                ref message_ids,
            } => (*channel_id, message_ids.to_owned()),
            State::NotSet => return,
        },
        None => return,
    };

    // Ignore the reaction if it isn't on an rr message
    if !(reaction.channel_id == rr_channel_id
        && rr_message_ids.contains(reaction.message_id.as_u64()))
    {
        return;
    }

    info!("On correct message reacted!");
    if let ReactionType::Unicode(ref s) = reaction.emoji {
        // check if rr registered for this emoji
        let results = ReactionRole::list_by_emoji(&get_client(&ctx).await.unwrap(), s)
            .await
            .expect("Could not get by emojis");

        if let Some(reaction_role) = results.first() {
            info!("Found reaction role for {}!", reaction.emoji);

            let guild = reaction
                .guild_id
                .expect("Got an rr reaction without a guild id");
            let mut member = guild
                .member(&ctx, reaction.user_id.unwrap())
                .await
                .expect("Couldn't get member that reacted to rr message");
            let role_id = reaction_role.role_id;
            let has_role = member.roles.contains(&(role_id as u64).into());

            if reaction_added == has_role {
                return;
            }

            if reaction_added {
                member.add_role(&ctx, role_id as u64).await.unwrap();
            } else {
                member.remove_role(&ctx, role_id as u64).await.unwrap();
            }

            let guild_name = guild.name(&ctx).await.expect("Couldn't get guild name");

            member
                .user
                .direct_message(ctx, |m| {
                    m.content(format!(
                        "{} role **{}** ({}) in **{}**",
                        if reaction_added { "Added" } else { "Removed" },
                        reaction_role.role_name,
                        reaction.emoji,
                        guild_name
                    ))
                })
                .await
                .expect("Couldn't notify user about reaction role");
        }
    }
}
