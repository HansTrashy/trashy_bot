use serenity::{
    framework::standard::{Args, CommandResult, macros::command},
    model::channel::Message,
    model::id::RoleId,
    model::id::ChannelId,
};
use serenity::model::gateway::Activity;
use serenity::model::user::OnlineStatus;
use serenity::prelude::*;
use log::*;
use crate::models::server_config::{ServerConfig, NewServerConfig};
use crate::models::mute::{Mute, NewMute};
use serde::{Deserialize, Serialize};
use crate::schema::server_configs;
use crate::schema::mutes;
use crate::DatabaseConnection;
use diesel::prelude::*;
use super::config::GuildConfig;
use chrono::{DateTime, Utc};
use crate::SchedulerKey;
use time::Duration;
use white_rabbit::{Scheduler, DateResult};
use crate::util;

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
        .get_mut::<SchedulerKey>()
        .expect("Expected Scheduler.")
        .clone();
    let mut scheduler = scheduler.write();

    let duration = util::parse_duration(&args.single::<String>()?);
    let mute_message = args.single::<String>()?;

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
                        for user in &msg.mentions {
                            match guild_id.member(&ctx, user) {
                                Ok(mut member) => {
                                    let _ = member.add_role(&ctx, RoleId(*mute_role));
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
                                end_time: end_time.clone(),
                            })
                            .collect::<Vec<NewMute>>();
                        diesel::insert_into(mutes::table)
                            .values(&mutes)
                            .execute(&*conn)?;

                        let http = ctx.http.clone();
                        let data_clone = ctx.data.clone();
                        let msg_clone = msg.clone();
                        let mute_role_clone = mute_role.clone();

                        scheduler.add_task_duration(duration, move |_| {
                            let conn_clone = match data_clone.write().get::<DatabaseConnection>() {
                                Some(v) => v.get().unwrap(),
                                None => panic!("Failed to get database connection"),
                            };
                            for user in &msg_clone.mentions {
                                match guild_id.member(&*http, user) {
                                    Ok(mut member) => {
                                        let _ = member.remove_role(&http, RoleId(mute_role_clone));
                                    }
                                    Err(e) => error!("could not get member: {:?}", e),
                                };
                            }
                            let user_ids = msg_clone
                                .mentions
                                .iter()
                                .map(|user| *user.id.as_u64() as i64)
                                .collect::<Vec<i64>>();
                            diesel::delete(
                                mutes::table
                                    .filter(mutes::server_id.eq(*guild_id.as_u64() as i64))
                                    .filter(mutes::user_id.eq_any(user_ids)),
                            )
                            .execute(&*conn_clone)
                            .expect("could not delete mute");

                            DateResult::Done
                        });
                    }

                    if let Some(modlog_channel) = &guild_config.modlog_channel {
                        let _ = ChannelId(*modlog_channel).send_message(&ctx, |m| {
                            m.embed(|e| {
                                e.description(format!(
                                    "Muted users: {:?} for {} seconds because: {}",
                                    &msg.mentions, duration, mute_message,
                                ))
                                .color((0, 120, 220))
                            })
                        });
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

#[command]
#[only_in("guilds")]
#[allowed_roles("Mods")]
pub fn unmute(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let mut data = ctx.data.write();
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
                    for user in &msg.mentions {
                        match guild_id.member(&ctx, user) {
                            Ok(mut member) => {
                                let _ = member.remove_role(&ctx, RoleId(*mute_role));
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
                }

                if let Some(modlog_channel) = &guild_config.modlog_channel {
                    let _ = ChannelId(*modlog_channel).send_message(&ctx, |m| {
                        m.embed(|e| {
                            e.description(format!("Unmuted users: {:?}", &msg.mentions))
                                .color((0, 120, 220))
                        })
                    });
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
#[allowed_roles("Mods")]
pub fn kick(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let kick_message = args.single::<String>()?;

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
                    for user in &msg.mentions {
                        let member = guild_id.member(&ctx, user)?;
                        let _ = member.kick(&ctx);
                    }
                }

                if let Some(modlog_channel) = &guild_config.modlog_channel {
                    let _ = ChannelId(*modlog_channel).send_message(&ctx, |m| {
                        m.embed(|e| {
                            e.description(format!(
                                "Kicked users: {:?} because: {}",
                                &msg.mentions, kick_message
                            ))
                            .color((0, 120, 220))
                        })
                    });
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
#[allowed_roles("Mods")]
pub fn ban(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let days = args.single::<u8>()?;
    let ban_msg = args.single::<String>()?;
    let ban_msg: &str = &*ban_msg;

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
                    for user in &msg.mentions {
                        let member = guild_id.member(&ctx, user)?;
                        let _ = member.ban(&ctx, &(days, ban_msg));
                    }
                }

                if let Some(modlog_channel) = &guild_config.modlog_channel {
                    let _ = ChannelId(*modlog_channel).send_message(&ctx, |m| {
                        m.embed(|e| {
                            e.description(format!(
                                "Banned users: {:?} because: {}",
                                &msg.mentions, ban_msg
                            ))
                            .color((0, 120, 220))
                        })
                    });
                }
            }
            None => {
                let _ = msg.reply(&ctx, "server config missing");
            }
        }
    }

    Ok(())
}
