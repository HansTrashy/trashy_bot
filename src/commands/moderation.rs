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
use serde::{Deserialize, Serialize};
use crate::schema::server_configs;
use crate::DatabaseConnection;
use diesel::prelude::*;
use super::config::GuildConfig;

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
    let seconds = args.single::<u64>()?;
    let mute_message = args.single::<String>();

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
                        let mut member = guild_id.member(&ctx, user)?;
                        let _ = member.add_role(&ctx, RoleId(*mute_role));
                    }
                }

                if let Some(modlog_channel) = &guild_config.modlog_channel {
                    let _ = ChannelId(*modlog_channel).send_message(&ctx, |m| {
                        m.embed(|e| {
                            e.description(format!(
                                "Muted users: {:?} for {} seconds",
                                &msg.mentions, seconds,
                            ))
                            .color((0, 120, 220))
                        })
                    });
                }
            }
            None => {
                let _ = msg.reply(&ctx, "mute role not set");
            }
        }
    }

    Ok(())
}

#[command]
#[num_args(0)]
#[only_in("guilds")]
#[allowed_roles("Mods")]
pub fn unmute(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[num_args(0)]
#[only_in("guilds")]
#[allowed_roles("Mods")]
pub fn kick(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    Ok(())
}

#[command]
#[num_args(0)]
#[only_in("guilds")]
#[allowed_roles("Mods")]
pub fn ban(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    Ok(())
}
