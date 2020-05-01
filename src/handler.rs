use crate::commands::config::Guild;
use crate::commands::userinfo::UserInfo;
use crate::models::mute::Mute;
use crate::models::server_config::ServerConfig;
use crate::models::tag::Tag;
use crate::DatabasePool;
use chrono::Utc;
use serenity::{
    async_trait,
    model::{
        channel::Message,
        channel::Reaction,
        channel::ReactionType,
        gateway::{Activity, Ready},
        guild::Member,
        id::ChannelId,
        id::GuildId,
        id::RoleId,
        user::User,
    },
    prelude::*,
};
use tracing::{error, info, instrument};

mod fav;
mod reaction_roles;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Activity::listening("$help")).await;
        println!("{} is connected!", ready.user.name);
    }

    async fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, mut new_member: Member) {
        let conn = ctx
            .data
            .read()
            .await
            .get::<DatabasePool>()
            .expect("Failed to access db pool")
            .get()
            .await;

        if conn.is_err() {
            error!("Failed to get database connection");
            return;
        }
        let mut conn = conn.unwrap();

        if let Ok(mut config) = ServerConfig::get(&mut *conn, *guild_id.as_u64() as i64).await {
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
                let _ = ChannelId(userlog_channel)
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
                    .await;
            }

            let mute = Mute::get(
                &mut *conn,
                *guild_id.as_u64() as i64,
                *member_id.as_u64() as i64,
            )
            .await;

            if let Ok(_mute) = mute {
                if let Some(mute_role) = g_cfg.mute_role {
                    let _ = new_member.add_role(&ctx, RoleId(mute_role)).await;
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
        if let Some(pool) = ctx.data.read().await.get::<DatabasePool>() {
            let mut conn = pool.get().await.unwrap();

            if let Ok(mut config) = ServerConfig::get(&mut *conn, *guild_id.as_u64() as i64).await {
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
                    let _ = ChannelId(userlog_channel)
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
                        .await;
                }
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        match reaction.emoji {
            ReactionType::Unicode(ref s) if s.starts_with("📗") => {
                fav::add(ctx, reaction).await;
            }
            _ => {
                reaction_roles::add_role(ctx, reaction).await;
            }
        }
    }

    async fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        match removed_reaction.emoji {
            _ => reaction_roles::remove_role(ctx, removed_reaction).await,
        }
    }
}
