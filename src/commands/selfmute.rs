use super::config::Guild;
use crate::models::mute::Mute;
use crate::models::server_config::ServerConfig;
use crate::util;
use crate::util::get_client;
use crate::DatabasePool;
use chrono::{Duration, Utc};
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    model::id::RoleId,
    model::prelude::*,
};
use tokio::time::delay_for;
use tracing::error;

#[command]
#[num_args(1)]
#[description = "Mutes youself for the given duration supports (d, h, m, s)"]
#[usage = "*duration*"]
#[example = "1h"]
#[only_in("guilds")]
pub async fn selfmute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let duration = util::parse_duration(&args.single::<String>()?).expect("invalid duration");
    let pool = get_client(&ctx).await?;

    if duration > Duration::hours(24) || duration < Duration::seconds(60) {
        msg.reply(
            ctx,
            "You can not mute yourself for less than 60 seconds or more than 24 hours!",
        )
        .await?;
        return Ok(());
    }

    if let Some(guild_id) = msg.guild_id {
        match ServerConfig::get(&pool, *guild_id.as_u64() as i64).await {
            Ok(server_config) => {
                let guild_config: Guild = serde_json::from_value(server_config.config).unwrap();

                if let Some(mute_role) = &guild_config.mute_role {
                    match guild_id.member(ctx, msg.author.id).await {
                        Ok(mut member) => {
                            member.add_role(&ctx, RoleId(*mute_role)).await?;
                        }
                        Err(e) => error!("could not get member: {:?}", e),
                    };

                    let end_time = Utc::now() + duration;

                    Mute::create(
                        &pool,
                        *guild_id.as_u64() as i64,
                        *msg.author.id.as_u64() as i64,
                        end_time,
                    )
                    .await?;

                    msg.react(ctx, ReactionType::Unicode("✅".to_string()))
                        .await?;

                    delay_for(duration.to_std()?).await;

                    match guild_id.member(ctx, msg.author.id).await {
                        Ok(mut member) => {
                            member
                                .remove_role(&ctx, RoleId(*mute_role))
                                .await
                                .expect("could not remove role");
                        }
                        Err(e) => error!("could not get member: {:?}", e),
                    };

                    Mute::delete(
                        &pool,
                        *guild_id.as_u64() as i64,
                        *msg.author.id.as_u64() as i64,
                    )
                    .await?;
                }
            }
            Err(_e) => {
                msg.reply(ctx, "server config missing").await?;
            }
        }
    }

    Ok(())
}
