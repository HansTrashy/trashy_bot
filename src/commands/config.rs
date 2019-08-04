use serenity::{
    framework::standard::{Args, CommandResult, macros::command},
    model::channel::Message,
};
use serenity::model::gateway::Activity;
use serenity::model::user::OnlineStatus;
use serenity::prelude::*;
use log::*;
use crate::models::server_config::{ServerConfig, NewServerConfig};
use serde::{Deserialize, Serialize};
use crate::schema::server_configs;
use crate::DatabaseConnection;
use diesel::prelude::*;
use crate::SchedulerKey;

#[command]
#[num_args(0)]
#[allowed_roles("Mods")]
pub fn status(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    ctx.set_presence(Some(Activity::listening("$help")), OnlineStatus::Online);
    Ok(())
}

// Keep every setting optional and use reasonable defaults
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GuildConfig {
    pub modlog_channel: Option<u64>,
    pub mute_role: Option<u64>,
    pub userlog_channel: Option<u64>,
}

#[command]
#[num_args(0)]
#[allowed_roles("Mods")]
pub fn show_config(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let mut data = ctx.data.write();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };

    if let Some(server_id) = msg.guild_id {
        let server_config = server_configs::table
            .filter(server_configs::server_id.eq(*server_id.as_u64() as i64))
            .first::<ServerConfig>(&*conn)
            .optional()?;

        if let Some(server_config) = server_config {
            let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.description(format!("{:?}", &server_config))
                        .color((0, 120, 220))
                })
            });
        } else {
            let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.description("config for this server is not available")
                        .color((255, 0, 0))
                })
            });
        }
    }

    Ok(())
}

#[command]
#[num_args(1)]
#[allowed_roles("Mods")]
pub fn set_modlog(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let modlog_channel = args.parse::<u64>()?;
    let mut data = ctx.data.write();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };

    if let Some(server_id) = msg.guild_id {
        match server_configs::table
            .filter(server_configs::server_id.eq(*server_id.as_u64() as i64))
            .first::<ServerConfig>(&*conn)
            .optional()?
        {
            Some(mut config) => {
                let mut old_guild_config: GuildConfig =
                    serde_json::from_value(config.config.take()).unwrap();

                old_guild_config.modlog_channel = Some(modlog_channel);

                config.config = serde_json::to_value(old_guild_config).unwrap();

                let inserted_config = diesel::update(server_configs::table)
                    .set(&config)
                    .get_result::<ServerConfig>(&*conn)?;

                let _ = msg.channel_id.send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.description(format!("{:?}", &inserted_config))
                            .color((0, 120, 220))
                    })
                });
            }
            None => {
                let mut guild_config = GuildConfig::default();

                guild_config.modlog_channel = Some(modlog_channel);

                let new_server_config = NewServerConfig {
                    server_id: *server_id.as_u64() as i64,
                    config: serde_json::to_value(guild_config).unwrap(),
                };

                let inserted_config = diesel::insert_into(server_configs::table)
                    .values(&new_server_config)
                    .get_result::<ServerConfig>(&*conn)?;

                let _ = msg.channel_id.send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.description(format!("{:?}", &inserted_config))
                            .color((0, 120, 220))
                    })
                });
            }
        }
    }

    Ok(())
}

#[command]
#[num_args(1)]
#[allowed_roles("Mods")]
pub fn set_userlog(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let userlog_channel = args.parse::<u64>()?;
    let mut data = ctx.data.write();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };

    if let Some(server_id) = msg.guild_id {
        match server_configs::table
            .filter(server_configs::server_id.eq(*server_id.as_u64() as i64))
            .first::<ServerConfig>(&*conn)
            .optional()?
        {
            Some(mut config) => {
                let mut old_guild_config: GuildConfig =
                    serde_json::from_value(config.config.take()).unwrap();

                old_guild_config.userlog_channel = Some(userlog_channel);

                config.config = serde_json::to_value(old_guild_config).unwrap();

                let inserted_config = diesel::update(server_configs::table)
                    .set(&config)
                    .get_result::<ServerConfig>(&*conn)?;

                let _ = msg.channel_id.send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.description(format!("{:?}", &inserted_config))
                            .color((0, 120, 220))
                    })
                });
            }
            None => {
                let mut guild_config = GuildConfig::default();

                guild_config.modlog_channel = Some(userlog_channel);

                let new_server_config = NewServerConfig {
                    server_id: *server_id.as_u64() as i64,
                    config: serde_json::to_value(guild_config).unwrap(),
                };

                let inserted_config = diesel::insert_into(server_configs::table)
                    .values(&new_server_config)
                    .get_result::<ServerConfig>(&*conn)?;

                let _ = msg.channel_id.send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.description(format!("{:?}", &inserted_config))
                            .color((0, 120, 220))
                    })
                });
            }
        }
    }

    Ok(())
}

#[command]
#[num_args(1)]
#[allowed_roles("Mods")]
pub fn set_muterole(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let mute_role = args.parse::<u64>()?;
    let mut data = ctx.data.write();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };

    if let Some(server_id) = msg.guild_id {
        match server_configs::table
            .filter(server_configs::server_id.eq(*server_id.as_u64() as i64))
            .first::<ServerConfig>(&*conn)
            .optional()?
        {
            Some(mut config) => {
                let mut old_guild_config: GuildConfig =
                    serde_json::from_value(config.config.take()).unwrap();

                old_guild_config.mute_role = Some(mute_role);

                config.config = serde_json::to_value(old_guild_config).unwrap();

                let inserted_config = diesel::update(server_configs::table)
                    .set(&config)
                    .get_result::<ServerConfig>(&*conn)?;

                let _ = msg.channel_id.send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.description(format!("{:?}", &inserted_config))
                            .color((0, 120, 220))
                    })
                });
            }
            None => {
                let mut guild_config = GuildConfig::default();

                guild_config.mute_role = Some(mute_role);

                let new_server_config = NewServerConfig {
                    server_id: *server_id.as_u64() as i64,
                    config: serde_json::to_value(guild_config).unwrap(),
                };

                let inserted_config = diesel::insert_into(server_configs::table)
                    .values(&new_server_config)
                    .get_result::<ServerConfig>(&*conn)?;

                let _ = msg.channel_id.send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.description(format!("{:?}", &inserted_config))
                            .color((0, 120, 220))
                    })
                });
            }
        }
    }

    Ok(())
}
