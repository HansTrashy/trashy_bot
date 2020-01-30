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
use log::*;
use serde::{Deserialize, Serialize};
use serenity::model::gateway::Activity;
use serenity::model::user::OnlineStatus;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    model::id::ChannelId,
    model::id::RoleId,
    model::prelude::*,
};
use time::Duration;

#[command]
#[num_args(1)]
#[description = "Mutes youself for the given duration supports (w, d, h, m, s)"]
#[usage = "*duration*"]
#[example = "1h"]
#[only_in("guilds")]
pub fn selfmute(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
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

    let duration = util::parse_duration(&args.single::<String>()?).unwrap();

    if duration > Duration::hours(24) {
        let _ = msg.reply(&ctx, "You can not mute yourself for more than 24 hours!");
        return Ok(());
    }

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
                    match guild_id.member(&ctx, msg.author.id) {
                        Ok(mut member) => {
                            let _ = member.add_role(&ctx, RoleId(*mute_role));
                        }
                        Err(e) => error!("could not get member: {:?}", e),
                    };

                    let end_time = Utc::now() + duration;
                    let mute = NewMute {
                        server_id: *guild_id.as_u64() as i64,
                        user_id: *msg.author.id.as_u64() as i64,
                        end_time,
                    };
                    diesel::insert_into(mutes::table)
                        .values(&mute)
                        .execute(&*conn)?;

                    let task =
                        Task::remove_mute(*guild_id.as_u64(), *msg.author.id.as_u64(), *mute_role);
                    scheduler.add_task(duration, task);

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
