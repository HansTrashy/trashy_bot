use super::config::GuildConfig;
use crate::models::mute::{Mute, NewMute};
use crate::models::server_config::{NewServerConfig, ServerConfig};
use crate::scheduler::Task;
use crate::schema::mutes;
use crate::schema::server_configs;
use crate::util;
use crate::DatabaseConnection;
use crate::TrashyScheduler;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use itertools::Itertools;
use log::*;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    model::guild::Member,
    model::id::ChannelId,
    model::id::RoleId,
    model::prelude::*,
    model::user::User,
};
use time::Duration;

#[command]
#[only_in("guilds")]
#[allowed_roles("Mods")]
pub fn mute(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };

    let scheduler = data
        .get_mut::<TrashyScheduler>()
        .expect("Expected Scheduler.")
        .clone();

    let duration = util::parse_duration(&args.single::<String>()?);
    let mute_message = args.rest();

    if let Some(duration) = duration {
        if let Some(guild_id) = msg.guild_id {
            match server_configs::table
                .filter(server_configs::server_id.eq(*guild_id.as_u64() as i64))
                .first::<ServerConfig>(&*conn)
                .optional()?
            {
                Some(server_config) => {
                    let guild_config: GuildConfig =
                        serde_json::from_value(server_config.config).unwrap();

                    if let Some(mute_role) = &guild_config.mute_role {
                        let mut found_members = Vec::new();
                        for user in &msg.mentions {
                            match guild_id.member(&ctx, user) {
                                Ok(mut member) => {
                                    let _ = member.add_role(&ctx, RoleId(*mute_role));
                                    found_members.push(member);
                                }
                                Err(e) => error!("could not get member: {:?}", e),
                            };
                        }
                        let end_time = Utc::now() + duration;
                        let mutes = msg
                            .mentions
                            .iter()
                            .map(|user| NewMute {
                                server_id: *guild_id.as_u64() as i64,
                                user_id: *user.id.as_u64() as i64,
                                end_time,
                            })
                            .collect::<Vec<NewMute>>();
                        diesel::insert_into(mutes::table)
                            .values(&mutes)
                            .execute(&*conn)?;

                        for user in &msg.mentions {
                            let task = Task::remove_mute(
                                *guild_id.as_u64(),
                                *user.id.as_u64(),
                                *mute_role,
                            );
                            scheduler.add_task(duration, task);
                        }

                        if let Some(modlog_channel) = &guild_config.modlog_channel {
                            if !found_members.is_empty() {
                                let _ = ChannelId(*modlog_channel).send_message(&ctx, |m| {
                                    m.embed(|e| {
                                        e.description(create_mute_message(
                                            &found_members,
                                            &duration,
                                            &mute_message,
                                        ))
                                        .color((0, 120, 220))
                                    })
                                });
                            }
                        }

                        let _ = msg.react(&ctx, ReactionType::Unicode("✅".to_string()));
                    }
                }
                None => {
                    let _ = msg.reply(&ctx, "server config missing");
                }
            }
        }
    }

    Ok(())
}

fn create_mute_message(users: &[Member], duration: &Duration, mute_message: &str) -> String {
    let intro = if users.len() > 1 {
        "Muted users:"
    } else {
        "Muted user:"
    };
    let users = users
        .iter()
        .map(|u| {
            if let Some(nick) = &u.nick {
                format!(
                    "{} ({}#{})",
                    nick,
                    u.user.read().name,
                    u.user.read().discriminator
                )
            } else {
                format!("{}#{}", u.user.read().name, u.user.read().discriminator)
            }
        })
        .join(", ");
    format!(
        "{} **{}** for **{}**\nPlease note: *{}*",
        intro,
        users,
        util::humanize_duration(duration),
        mute_message
    )
}

fn create_ban_message(users: &[Member], ban_message: &str) -> String {
    let intro = if users.len() > 1 {
        "Banned users:"
    } else {
        "Banned user:"
    };
    let users = users
        .iter()
        .map(|u| {
            if let Some(nick) = &u.nick {
                format!(
                    "{} ({}#{})",
                    nick,
                    u.user.read().name,
                    u.user.read().discriminator
                )
            } else {
                format!("{}#{}", u.user.read().name, u.user.read().discriminator)
            }
        })
        .join(", ");
    format!("{} **{}**\nPlease note: *{}*", intro, users, ban_message)
}

fn create_kick_message(users: &[Member], kick_message: &str) -> String {
    let intro = if users.len() > 1 {
        "Kicked users:"
    } else {
        "Kicked user:"
    };
    let users = users
        .iter()
        .map(|u| {
            if let Some(nick) = &u.nick {
                format!(
                    "{} ({}#{})",
                    nick,
                    u.user.read().name,
                    u.user.read().discriminator
                )
            } else {
                format!("{}#{}", u.user.read().name, u.user.read().discriminator)
            }
        })
        .join(", ");
    format!("{} **{}**\nPlease note: *{}*", intro, users, kick_message)
}

fn create_unmute_message(users: &[Member]) -> String {
    let intro = if users.len() > 1 {
        "Unmuted users:"
    } else {
        "Unmuted user:"
    };
    let users = users
        .iter()
        .map(|u| {
            if let Some(nick) = &u.nick {
                format!(
                    "{} ({}#{})",
                    nick,
                    u.user.read().name,
                    u.user.read().discriminator
                )
            } else {
                format!("{}#{}", u.user.read().name, u.user.read().discriminator)
            }
        })
        .join(", ");
    format!("{} {}", intro, users)
}

#[command]
#[only_in("guilds")]
#[allowed_roles("Mods")]
pub fn unmute(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.write();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };

    if let Some(guild_id) = msg.guild_id {
        match server_configs::table
            .filter(server_configs::server_id.eq(*guild_id.as_u64() as i64))
            .first::<ServerConfig>(&*conn)
            .optional()?
        {
            Some(server_config) => {
                let guild_config: GuildConfig =
                    serde_json::from_value(server_config.config).unwrap();

                if let Some(mute_role) = &guild_config.mute_role {
                    let mut found_members = Vec::new();
                    for user in &msg.mentions {
                        match guild_id.member(&ctx, user) {
                            Ok(mut member) => {
                                let _ = member.remove_role(&ctx, RoleId(*mute_role));
                                found_members.push(member);
                            }
                            Err(e) => error!("could not get member: {:?}", e),
                        };
                    }
                    let user_ids = msg
                        .mentions
                        .iter()
                        .map(|user| *user.id.as_u64() as i64)
                        .collect::<Vec<i64>>();
                    diesel::delete(
                        mutes::table
                            .filter(mutes::server_id.eq(*guild_id.as_u64() as i64))
                            .filter(mutes::user_id.eq_any(user_ids)),
                    )
                    .execute(&*conn)?;

                    if let Some(modlog_channel) = &guild_config.modlog_channel {
                        if !found_members.is_empty() {
                            let _ = ChannelId(*modlog_channel).send_message(&ctx, |m| {
                                m.embed(|e| {
                                    e.description(create_unmute_message(&found_members))
                                        .color((0, 120, 220))
                                })
                            });
                        }
                    }

                    let _ = msg.react(&ctx, ReactionType::Unicode("✅".to_string()));
                }
            }
            None => {
                let _ = msg.reply(&ctx, "server config missing");
            }
        }
    }
    Ok(())
}

#[command]
#[only_in("guilds")]
#[aliases("yeet")]
#[allowed_roles("Mods")]
pub fn kick(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.write();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let kick_message = args.rest();

    if let Some(guild_id) = msg.guild_id {
        match server_configs::table
            .filter(server_configs::server_id.eq(*guild_id.as_u64() as i64))
            .first::<ServerConfig>(&*conn)
            .optional()?
        {
            Some(server_config) => {
                let guild_config: GuildConfig =
                    serde_json::from_value(server_config.config).unwrap();

                let mut found_members = Vec::new();
                for user in &msg.mentions {
                    let member = guild_id.member(&ctx, user)?;
                    let _ = member.kick(&ctx);
                    found_members.push(member);
                }

                if let Some(modlog_channel) = &guild_config.modlog_channel {
                    if !found_members.is_empty() {
                        let _ = ChannelId(*modlog_channel).send_message(&ctx, |m| {
                            m.embed(|e| {
                                e.description(create_kick_message(&found_members, &kick_message))
                                    .color((0, 120, 220))
                            })
                        });
                    }
                }

                let _ = msg.react(&ctx, ReactionType::Unicode("✅".to_string()));
            }
            None => {
                let _ = msg.reply(&ctx, "server config missing");
            }
        }
    }

    Ok(())
}

#[command]
#[only_in("guilds")]
#[allowed_roles("Mods")]
pub fn ban(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.write();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let ban_msg = args.rest();

    if let Some(guild_id) = msg.guild_id {
        match server_configs::table
            .filter(server_configs::server_id.eq(*guild_id.as_u64() as i64))
            .first::<ServerConfig>(&*conn)
            .optional()?
        {
            Some(server_config) => {
                let guild_config: GuildConfig =
                    serde_json::from_value(server_config.config).unwrap();

                let mut found_members = Vec::new();
                for user in &msg.mentions {
                    let member = guild_id.member(&ctx, user)?;
                    let _ = member.ban(&ctx, &(0, ban_msg));
                    found_members.push(member);
                }

                if let Some(modlog_channel) = &guild_config.modlog_channel {
                    if !found_members.is_empty() {
                        let _ = ChannelId(*modlog_channel).send_message(&ctx, |m| {
                            m.embed(|e| {
                                e.description(create_ban_message(&found_members, ban_msg))
                                    .color((0, 120, 220))
                            })
                        });
                    }
                }

                let _ = msg.react(&ctx, ReactionType::Unicode("✅".to_string()));
            }
            None => {
                let _ = msg.reply(&ctx, "server config missing");
            }
        }
    }

    Ok(())
}
