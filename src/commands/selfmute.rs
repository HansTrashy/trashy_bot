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
        .get_mut::<SchedulerKey>()
        .expect("Expected Scheduler.")
        .clone();
    let mut scheduler = scheduler.write();

    let duration = util::parse_duration(&args.single::<String>()?).unwrap();

    if duration.num_hours() > 24 {
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
                        end_time: end_time.clone(),
                    };
                    diesel::insert_into(mutes::table)
                        .values(&mute)
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

                        match guild_id.member(&*http, msg_clone.author.id) {
                            Ok(mut member) => {
                                let _ = member.remove_role(&http, RoleId(mute_role_clone));
                            }
                            Err(e) => error!("could not get member: {:?}", e),
                        };

                        diesel::delete(
                            mutes::table
                                .filter(mutes::server_id.eq(*guild_id.as_u64() as i64))
                                .filter(mutes::user_id.eq(*msg_clone.author.id.as_u64() as i64)),
                        )
                        .execute(&*conn_clone)
                        .expect("could not delete mute");

                        DateResult::Done
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
