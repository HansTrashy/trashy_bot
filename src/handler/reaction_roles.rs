use crate::interaction::wait::Action;
use crate::interaction::wait::WaitEvent;
use crate::models::reaction_role::ReactionRole;
use crate::schema::reaction_roles::dsl::*;
use crate::DatabaseConnection;
use crate::Waiter;
use chrono::prelude::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use log::info;
use serenity::{
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        help_commands, Args, CommandOptions, DispatchError, HelpBehaviour, StandardFramework,
    },
    model::{
        channel::Message, channel::Reaction, channel::ReactionType, gateway::Ready, id::GuildId,
        id::RoleId, Permissions,
    },
    prelude::*,
    utils::{content_safe, ContentSafeOptions},
};

pub fn add_role(ctx: Context, add_reaction: Reaction) {
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.clone(),
        None => return,
    };
    // check if reaction is on rr message
    if add_reaction.channel_id == 553508425745563648
        && add_reaction.message_id == 559851083422498836
    {
        info!("On correct message reacted!");
        if let ReactionType::Unicode(ref s) = add_reaction.emoji {
            // check if rr registered for this emoji
            let results = reaction_roles
                .filter(emoji.eq(s))
                .load::<ReactionRole>(&*conn.lock())
                .expect("could not load reaction roles");

            if !results.is_empty() {
                info!("Found role for this emoji!");
                // got reaction role, add it to user!
                let guild_id = GuildId::from(553329127705411614);
                if let Ok(mut member) = guild_id.member(add_reaction.user_id) {
                    let rs = member.add_role(RoleId::from(165575458954543104));
                    // info!("{:?}", member);
                    // let r_id = results[0].role_id as u64;
                    // let rs = member.add_role(results[0].role_id as u64);
                    info!("{:?}", rs);
                }
            }
        }
    }
}

pub fn remove_role(ctx: Context, remove_reaction: Reaction) {
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.clone(),
        None => return,
    };
    // check if reaction is on rr message
    if remove_reaction.channel_id == 553508425745563648
        && remove_reaction.message_id == 559851083422498836
    {
        info!("On correct message reacted!");
        if let ReactionType::Unicode(ref s) = remove_reaction.emoji {
            // check if rr registered for this emoji
            let results = reaction_roles
                .filter(emoji.eq(s))
                .load::<ReactionRole>(&*conn.lock())
                .expect("could not load reaction roles");

            if !results.is_empty() {
                info!("Found role for this emoji!");
                // got reaction role, add it to user!
                let guild_id = GuildId::from(553329127705411614);
                if let Ok(mut member) = guild_id.member(remove_reaction.user_id) {
                    info!("{:?}", member);
                    let rs = member.remove_role(results[0].role_id as u64);
                    info!("{:?}", rs);
                }
            }
        }
    }
}
