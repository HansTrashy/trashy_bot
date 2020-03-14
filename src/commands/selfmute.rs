use super::config::Guild;
use crate::models::mute::Mute;
use crate::models::server_config::ServerConfig;
use crate::scheduler::Task;
use crate::util;
use crate::DatabasePool;
use crate::TrashyScheduler;
use chrono::{Duration, Utc};
use log::*;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    model::id::RoleId,
    model::prelude::*,
};

#[command]
#[num_args(1)]
#[description = "Mutes youself for the given duration supports (w, d, h, m, s)"]
#[usage = "*duration*"]
#[example = "1h"]
#[only_in("guilds")]
pub async fn selfmute(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let mut conn = data
        .get::<DatabasePool>()
        .ok_or("Could not retrieve the database connection!")?
        .get()
        .await?;

    let scheduler = data
        .get_mut::<TrashyScheduler>()
        .expect("Expected Scheduler.")
        .clone();

    let duration = util::parse_duration(&args.single::<String>().await?).unwrap();

    if duration > Duration::hours(24) {
        let _ = msg.reply(&ctx, "You can not mute yourself for more than 24 hours!");
        return Ok(());
    }

    if let Some(guild_id) = msg.guild_id {
        match ServerConfig::get(&mut *conn, *guild_id.as_u64() as i64).await {
            Ok(server_config) => {
                let guild_config: Guild = serde_json::from_value(server_config.config).unwrap();

                if let Some(mute_role) = &guild_config.mute_role {
                    match guild_id.member(&ctx, msg.author.id).await {
                        Ok(mut member) => {
                            let _ = member.add_role(&ctx, RoleId(*mute_role));
                        }
                        Err(e) => error!("could not get member: {:?}", e),
                    };

                    let end_time = Utc::now() + duration;

                    Mute::create(
                        &mut *conn,
                        *guild_id.as_u64() as i64,
                        *msg.author.id.as_u64() as i64,
                        end_time,
                    )
                    .await?;

                    let task =
                        Task::remove_mute(*guild_id.as_u64(), *msg.author.id.as_u64(), *mute_role);
                    scheduler.add_task(duration, task);

                    let _ = msg.react(&ctx, ReactionType::Unicode("✅".to_string()));
                }
            }
            Err(_e) => {
                let _ = msg.reply(&ctx, "server config missing");
            }
        }
    }

    Ok(())
}
