use super::config::Guild;
use crate::models::mute::Mute;
use crate::models::server_config::ServerConfig;
use crate::scheduler::Task;
use crate::util;
use crate::DatabasePool;
use crate::TrashyScheduler;
use chrono::{Duration, Utc};
use futures::{stream, StreamExt};
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    model::guild::Member,
    model::id::ChannelId,
    model::id::RoleId,
    model::prelude::*,
};
use tracing::error;

#[command]
#[only_in("guilds")]
#[allowed_roles("Mods")]
pub async fn mute(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let pool = ctx.data.read().await
        .get::<DatabasePool>()
        .map(|p| p.clone())
        .ok_or("Could not retrieve the database connection!")?;
    let mut conn = pool.get().await?;

    let scheduler = ctx.data.write().await
        .get_mut::<TrashyScheduler>()
        .expect("Expected Scheduler.")
        .clone();

    let duration = util::parse_duration(&args.single::<String>()?);
    let mute_message = args.rest();

    if let Some(duration) = duration {
        if let Some(guild_id) = msg.guild_id {
            match ServerConfig::get(&mut *conn, *guild_id.as_u64() as i64).await {
                Ok(server_config) => {
                    let guild_config: Guild = serde_json::from_value(server_config.config).unwrap();

                    if let Some(mute_role) = &guild_config.mute_role {
                        let mut found_members = Vec::new();
                        for user in &msg.mentions {
                            match guild_id.member(&ctx, user).await {
                                Ok(mut member) => {
                                    let _ = member.add_role(&ctx, RoleId(*mute_role)).await;
                                    found_members.push(member);
                                }
                                Err(e) => error!("could not get member: {:?}", e),
                            };
                        }
                        let end_time = Utc::now() + duration;

                        for user in &msg.mentions {
                            let _ = Mute::create(
                                &mut *conn,
                                *guild_id.as_u64() as i64,
                                *user.id.as_u64() as i64,
                                end_time,
                            )
                            .await;
                        }

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
                                let description =
                                    create_mute_message(&found_members, &duration, &mute_message)
                                        .await;
                                let _ = ChannelId(*modlog_channel)
                                    .send_message(&ctx, |m| {
                                        m.embed(|e| e.description(description).color((0, 120, 220)))
                                    })
                                    .await;
                            }
                        }

                        msg.react(&ctx, ReactionType::Unicode("✅".to_string()))
                            .await;
                    }
                }
                Err(_e) => {
                    msg.reply(&ctx, "server config missing").await;
                }
            }
        }
    }

    Ok(())
}

async fn create_mute_message(users: &[Member], duration: &Duration, mute_message: &str) -> String {
    let intro = if users.len() > 1 {
        "Muted users:"
    } else {
        "Muted user:"
    };

    let users = stream::iter(users.iter())
        .map(|u| async move {
            let user = &u.user;
            if let Some(nick) = &u.nick {
                format!("{} ({}#{})", nick, user.name, user.discriminator)
            } else {
                format!("{}#{}", user.name, user.discriminator)
            }
        })
        .fold(String::new(), |mut acc, c| async move {
            if acc.is_empty() {
                acc.push_str(&c.await);
            } else {
                acc.push_str(", ");
                acc.push_str(&c.await);
            }
            acc
        })
        .await;
    format!(
        "{} **{}** for **{}**\nPlease note: *{}*",
        intro,
        users,
        util::humanize_duration(duration),
        mute_message
    )
}

async fn create_ban_message(users: &[Member], ban_message: &str) -> String {
    let intro = if users.len() > 1 {
        "Banned users:"
    } else {
        "Banned user:"
    };
    let users = stream::iter(users.iter())
        .map(|u| async move {
            let user = &u.user;
            if let Some(nick) = &u.nick {
                format!("{} ({}#{})", nick, user.name, user.discriminator)
            } else {
                format!("{}#{}", user.name, user.discriminator)
            }
        })
        .fold(String::new(), |mut acc, c| async move {
            if acc.is_empty() {
                acc.push_str(&c.await);
            } else {
                acc.push_str(", ");
                acc.push_str(&c.await);
            }
            acc
        })
        .await;
    format!("{} **{}**\nPlease note: *{}*", intro, users, ban_message)
}

async fn create_kick_message(users: &[Member], kick_message: &str) -> String {
    let intro = if users.len() > 1 {
        "Kicked users:"
    } else {
        "Kicked user:"
    };
    let users = stream::iter(users.iter())
        .map(|u| async move {
            let user = &u.user;
            if let Some(nick) = &u.nick {
                format!("{} ({}#{})", nick, user.name, user.discriminator)
            } else {
                format!("{}#{}", user.name, user.discriminator)
            }
        })
        .fold(String::new(), |mut acc, c| async move {
            if acc.is_empty() {
                acc.push_str(&c.await);
            } else {
                acc.push_str(", ");
                acc.push_str(&c.await);
            }
            acc
        })
        .await;
    format!("{} **{}**\nPlease note: *{}*", intro, users, kick_message)
}

async fn create_unmute_message(users: &[Member]) -> String {
    let intro = if users.len() > 1 {
        "Unmuted users:"
    } else {
        "Unmuted user:"
    };
    let users = stream::iter(users.iter())
        .map(|u| async move {
            let user = &u.user;
            if let Some(nick) = &u.nick {
                format!("{} ({}#{})", nick, user.name, user.discriminator)
            } else {
                format!("{}#{}", user.name, user.discriminator)
            }
        })
        .fold(String::new(), |mut acc, c| async move {
            if acc.is_empty() {
                acc.push_str(&c.await);
            } else {
                acc.push_str(", ");
                acc.push_str(&c.await);
            }
            acc
        })
        .await;
    format!("{} {}", intro, users)
}

#[command]
#[only_in("guilds")]
#[allowed_roles("Mods")]
pub async fn unmute(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let pool = ctx.data.read().await
        .get::<DatabasePool>()
        .map(|p| p.clone())
        .ok_or("Could not retrieve the database connection!")?;
    let mut conn = pool.get().await?;

    if let Some(guild_id) = msg.guild_id {
        match ServerConfig::get(&mut *conn, *guild_id.as_u64() as i64).await {
            Ok(server_config) => {
                let guild_config: Guild = serde_json::from_value(server_config.config).unwrap();

                if let Some(mute_role) = &guild_config.mute_role {
                    let mut found_members = Vec::new();
                    for user in &msg.mentions {
                        match guild_id.member(&ctx, user).await {
                            Ok(mut member) => {
                                let _ = member.remove_role(&ctx, RoleId(*mute_role));
                                found_members.push(member);
                            }
                            Err(e) => error!("could not get member: {:?}", e),
                        };
                    }

                    for user in &msg.mentions {
                        //TODO: this should be done in a single statement
                        let _ = Mute::delete(
                            &mut *conn,
                            *guild_id.as_u64() as i64,
                            *user.id.as_u64() as i64,
                        );
                    }

                    if let Some(modlog_channel) = &guild_config.modlog_channel {
                        if !found_members.is_empty() {
                            let description = create_unmute_message(&found_members).await;
                            let _ = ChannelId(*modlog_channel).send_message(&ctx, |m| {
                                m.embed(|e| e.description(description).color((0, 120, 220)))
                            });
                        }
                    }

                    msg.react(&ctx, ReactionType::Unicode("✅".to_string()))
                        .await?;
                }
            }
            Err(_e) => {
                msg.reply(&ctx, "server config missing").await?;
            }
        }
    }
    Ok(())
}

#[command]
#[only_in("guilds")]
#[aliases("yeet")]
#[allowed_roles("Mods")]
pub async fn kick(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let pool = ctx.data.read().await
        .get::<DatabasePool>()
        .map(|p| p.clone())
        .ok_or("Could not retrieve the database connection!")?;
    let mut conn = pool.get().await?;

    let kick_message = args.rest();

    if let Some(guild_id) = msg.guild_id {
        match ServerConfig::get(&mut *conn, *guild_id.as_u64() as i64).await {
            Ok(server_config) => {
                let guild_config: Guild = serde_json::from_value(server_config.config).unwrap();

                let mut found_members = Vec::new();
                for user in &msg.mentions {
                    let member = guild_id.member(&ctx, user).await?;
                    let _ = member.kick(&ctx).await;
                    found_members.push(member);
                }

                if let Some(modlog_channel) = &guild_config.modlog_channel {
                    if !found_members.is_empty() {
                        let description = create_kick_message(&found_members, &kick_message).await;
                        let _ = ChannelId(*modlog_channel)
                            .send_message(&ctx, |m| {
                                m.embed(|e| e.description(description).color((0, 120, 220)))
                            })
                            .await;
                    }
                }

                let _ = msg
                    .react(&ctx, ReactionType::Unicode("✅".to_string()))
                    .await;
            }
            Err(_e) => {
                let _ = msg.reply(&ctx, "server config missing").await;
            }
        }
    }

    Ok(())
}

#[command]
#[only_in("guilds")]
#[allowed_roles("Mods")]
pub async fn ban(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let pool = ctx.data.read().await
        .get::<DatabasePool>()
        .map(|p| p.clone())
        .ok_or("Could not retrieve the database connection!")?;
    let mut conn = pool.get().await?;

    let ban_msg = args.rest();

    if let Some(guild_id) = msg.guild_id {
        match ServerConfig::get(&mut *conn, *guild_id.as_u64() as i64).await {
            Ok(server_config) => {
                let guild_config: Guild = serde_json::from_value(server_config.config).unwrap();

                let mut found_members = Vec::new();
                for user in &msg.mentions {
                    let member = guild_id.member(&ctx, user).await?;
                    let _ = member.ban(&ctx, &(0, ban_msg)).await;
                    found_members.push(member);
                }

                if let Some(modlog_channel) = &guild_config.modlog_channel {
                    if !found_members.is_empty() {
                        let description = create_ban_message(&found_members, ban_msg).await;
                        let _ = ChannelId(*modlog_channel)
                            .send_message(&ctx, |m| {
                                m.embed(|e| e.description(description).color((0, 120, 220)))
                            })
                            .await;
                    }
                }

                let _ = msg
                    .react(&ctx, ReactionType::Unicode("✅".to_string()))
                    .await;
            }
            Err(_e) => {
                let _ = msg.reply(&ctx, "server config missing").await;
            }
        }
    }

    Ok(())
}
