mod fav;

use crate::commands::config::Guild;
use crate::commands::userinfo::UserInfo;
use crate::models::mute::Mute;
use crate::models::server_config::ServerConfig;
use crate::util::get_client;
use chrono::Utc;
use serenity::{
    async_trait,
    model::{
        channel::Reaction,
        channel::{ChannelType, Message, ReactionType},
        gateway::{Activity, Ready},
        guild::Member,
        id::ChannelId,
        id::GuildId,
        id::RoleId,
        user::User,
    },
    prelude::*,
    utils::MessageBuilder,
};
use tracing::info;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Activity::listening("$help")).await;

        tokio::spawn(async move {
            let mut thread_message: Option<Message> = None;
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            loop {
                if let Some(guild) = GuildId(217015995385118721).to_guild_cached(&ctx) {
                    let mut active_threads = String::new();
                    for channel in guild.threads {
                        tracing::info!(kind = ?channel.kind, name = ?channel.name(), meta = ?channel.thread_metadata, "CHANNEL");

                        if channel.kind == ChannelType::PublicThread {
                            if let Some(meta) = channel.thread_metadata {
                                if !meta.archived && !meta.locked {
                                    active_threads.push_str(&format!(
                                        "User: {:02}+ | Messages: {:02}+| {} \n",
                                        channel.member_count.unwrap_or(0),
                                        channel.message_count.unwrap_or(0),
                                        MessageBuilder::new().mention(&channel).build(),
                                    ));
                                }
                            }
                        }
                    }
                    match thread_message {
                        Some(ref mut msg) => {
                            tracing::info!("message is posted already, editing...");
                            std::mem::drop(msg.edit(&ctx, |m| m.content(active_threads)).await);
                        }
                        None => {
                            tracing::info!("message is not posted already, posting...");
                            thread_message = ChannelId(279934703904227328)
                                .send_message(&ctx, |m| m.content(active_threads))
                                .await
                                .ok();
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await;
            }
        });
        info!("{} is connected!", ready.user.name);
    }

    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, mut new_member: Member) {
        let pool = get_client(&ctx).await.unwrap();

        if let Ok(mut config) = ServerConfig::get(&pool, *guild_id.as_u64() as i64).await {
            let g_cfg: Guild = serde_json::from_value(config.config.take()).unwrap();

            let user_info = UserInfo {
                created_at: new_member
                    .user
                    .created_at()
                    .format("%d.%m.%Y %H:%M:%S")
                    .to_string(),
                created_at_ago: Utc::now()
                    .signed_duration_since(new_member.user.created_at())
                    .num_days(),
                member: None,
            };

            let information_body = format!(
                "**Joined discord:** {} ({} days ago)\n\n**Has joined this server**",
                user_info.created_at, user_info.created_at_ago,
            );

            let member_id = new_member.user.id;
            if let Some(userlog_channel) = g_cfg.userlog_channel {
                let member_name = new_member.user.name.to_string();
                let member_discriminator = new_member.user.discriminator;
                let member_avatar = new_member.user.static_avatar_url().unwrap_or_default();
                std::mem::drop(
                    ChannelId(userlog_channel)
                        .send_message(&ctx, |m| {
                            m.embed(|e| {
                                e.author(|a| a.name(&member_name).icon_url(member_avatar))
                                    .color((0, 220, 0))
                                    .description(&information_body)
                                    .footer(|f| {
                                        f.text(&format!(
                                            "{}#{} | id: {}",
                                            member_name, member_discriminator, member_id,
                                        ))
                                    })
                            })
                        })
                        .await,
                );
            }

            let mute =
                Mute::get(&pool, *guild_id.as_u64() as i64, *member_id.as_u64() as i64).await;

            if let Ok(_mute) = mute {
                if let Some(mute_role) = g_cfg.mute_role {
                    std::mem::drop(new_member.add_role(&ctx, RoleId(mute_role)).await);
                }
            }
        }
    }

    async fn guild_member_removal(
        &self,
        ctx: Context,
        guild_id: GuildId,
        user: User,
        _old_member: Option<Member>,
    ) {
        let pool = get_client(&ctx).await.unwrap();
        if let Ok(mut config) = ServerConfig::get(&pool, *guild_id.as_u64() as i64).await {
            let g_cfg: Guild = serde_json::from_value(config.config.take()).unwrap();

            let user_info = UserInfo {
                created_at: user.created_at().format("%d.%m.%Y %H:%M:%S").to_string(),
                created_at_ago: Utc::now()
                    .signed_duration_since(user.created_at())
                    .num_days(),
                member: None,
            };

            let information_body = format!(
                "**Joined discord:** {} ({} days ago)\n\n**Has left the server.**",
                user_info.created_at, user_info.created_at_ago,
            );

            if let Some(userlog_channel) = g_cfg.userlog_channel {
                std::mem::drop(
                    ChannelId(userlog_channel)
                        .send_message(&ctx, |m| {
                            m.embed(|e| {
                                e.author(|a| {
                                    a.name(&user.name)
                                        .icon_url(&user.static_avatar_url().unwrap_or_default())
                                })
                                .color((220, 0, 0))
                                .description(&information_body)
                                .footer(|f| {
                                    f.text(&format!(
                                        "{}#{} | id: {}",
                                        user.name, user.discriminator, &user.id,
                                    ))
                                })
                            })
                        })
                        .await,
                );
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        match reaction.emoji {
            ReactionType::Unicode(ref s) if s.starts_with('📗') => {
                fav::add(ctx, reaction).await;
            }
            _ => (),
        }
    }
}
