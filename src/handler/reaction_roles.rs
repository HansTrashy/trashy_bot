use crate::models::reaction_role::ReactionRole;
use crate::reaction_roles::State;
use crate::schema::reaction_roles::dsl::*;
use crate::DatabaseConnection;
use crate::ReactionRolesState;
use diesel::prelude::*;
use log::info;
use serenity::{
    model::{channel::Reaction, channel::ReactionType},
    prelude::*,
};

pub fn add_role(ctx: Context, add_reaction: Reaction) {
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => return,
    };
    let (rr_channel_id, rr_message_ids) = match data.get::<ReactionRolesState>() {
        Some(v) => match *v.lock() {
            State::Set {
                ref channel_id,
                ref message_ids,
            } => (*channel_id, message_ids.to_owned()),
            State::NotSet => return,
        },
        None => return,
    };
    // check if reaction is on rr message
    if add_reaction.channel_id == rr_channel_id
        && rr_message_ids.contains(add_reaction.message_id.as_u64())
    {
        info!("On correct message reacted!");
        if let ReactionType::Unicode(ref s) = add_reaction.emoji {
            // check if rr registered for this emoji
            let results = reaction_roles
                .filter(emoji.eq(s))
                .load::<ReactionRole>(&conn)
                .expect("could not load reaction roles");

            if !results.is_empty() {
                info!("Found role for this emoji!");
                if let Some(guild) = add_reaction
                    .channel_id
                    .to_channel()
                    .ok()
                    .and_then(|c| c.guild())
                {
                    if let Ok(mut member) = guild.read().guild_id.member(add_reaction.user_id) {
                        let _ = member.add_role(results[0].role_id as u64);
                    }
                }
            }
        }
    }
}

pub fn remove_role(ctx: Context, remove_reaction: Reaction) {
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => return,
    };
    let (rr_channel_id, rr_message_ids) = match data.get::<ReactionRolesState>() {
        Some(v) => match *v.lock() {
            State::Set {
                ref channel_id,
                ref message_ids,
            } => (*channel_id, message_ids.to_owned()),
            State::NotSet => return,
        },
        None => return,
    };
    // check if reaction is on rr message
    if remove_reaction.channel_id == rr_channel_id
        && rr_message_ids.contains(remove_reaction.message_id.as_u64())
    {
        info!("On correct message reacted!");
        if let ReactionType::Unicode(ref s) = remove_reaction.emoji {
            // check if rr registered for this emoji
            let results = reaction_roles
                .filter(emoji.eq(s))
                .load::<ReactionRole>(&conn)
                .expect("could not load reaction roles");

            if !results.is_empty() {
                info!("Found role for this emoji!");
                if let Some(guild) = remove_reaction
                    .channel_id
                    .to_channel()
                    .ok()
                    .and_then(|c| c.guild())
                {
                    if let Ok(mut member) = guild.read().guild_id.member(remove_reaction.user_id) {
                        let _ = member.remove_role(results[0].role_id as u64);
                    }
                }
            }
        }
    }
}
