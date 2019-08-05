use crate::interaction::wait::Action;
use crate::models::tag::NewTag;
use crate::DatabaseConnection;
use crate::Waiter;
use diesel::prelude::*;
use log::*;
use serenity::{
    model::{
        channel::Message, channel::Reaction, channel::ReactionType, gateway::Ready, id::GuildId,
        user::User, guild::Member, id::ChannelId, id::RoleId,
    },
    prelude::*,
};
use crate::schema::server_configs;
use crate::schema::mutes;
use crate::models::server_config::{ServerConfig, NewServerConfig};
use crate::models::mute::Mute;
use crate::commands::config::GuildConfig;
use chrono::Utc;
use crate::commands::userinfo::{UserInfo, MemberInfo};

mod blackjack;
mod fav;
mod reaction_roles;

pub struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, mut new_member: Member) {
        let mut data = ctx.data.write();

        if let Some(pool) = data.get::<DatabaseConnection>() {
            let conn = pool.get().unwrap();

            if let Some(mut config) = server_configs::table
                .filter(server_configs::server_id.eq(*guild_id.as_u64() as i64))
                .first::<ServerConfig>(&*conn)
                .optional()
                .unwrap()
            {
                let g_cfg: GuildConfig = serde_json::from_value(config.config.take()).unwrap();

                let mut user_info = UserInfo {
                    created_at: new_member
                        .user
                        .read()
                        .created_at()
                        .format("%d.%m.%Y %H:%M:%S")
                        .to_string(),
                    created_at_ago: Utc::now()
                        .signed_duration_since(new_member.user.read().created_at())
                        .num_days(),
                    member: None,
                };

                let default = "Unknown".to_string();

                let information_body = format!(
                    "**Joined discord:** {} ({} days ago)\n\n**Joined this server:** {} ({} days ago)\n\n**Roles:** {}",
                    user_info.created_at,
                    user_info.created_at_ago,
                    user_info
                        .member
                        .as_ref()
                        .and_then(|m| Some(&m.joined_at))
                        .unwrap_or(&default),
                    user_info
                        .member
                        .as_ref()
                        .and_then(|m| Some(&m.joined_at_ago))
                        .unwrap_or(&default),
                    user_info
                        .member
                        .as_ref()
                        .and_then(|m| Some(m.roles.join(", ")))
                        .unwrap_or_else(|| default.clone()),
                );

                if let Some(userlog_channel) = g_cfg.userlog_channel {
                    let _ = ChannelId(userlog_channel).send_message(&ctx, |m| {
                        m.embed(|e| {
                            e.author(|a| {
                                a.name(&new_member.user.read().name).icon_url(
                                    &new_member
                                        .user
                                        .read()
                                        .static_avatar_url()
                                        .unwrap_or_default(),
                                )
                            })
                            .color((0, 120, 220))
                            .description(&information_body)
                            .footer(|f| {
                                f.text(&format!(
                                    "{}#{} | id: {}",
                                    new_member.user.read().name,
                                    new_member.user.read().discriminator,
                                    &new_member.user.read().id,
                                ))
                            })
                        })
                    });
                }

                let mute = mutes::table
                    .filter(mutes::user_id.eq(*new_member.user.read().id.as_u64() as i64))
                    .first::<Mute>(&*conn)
                    .optional()
                    .unwrap();

                if let Some(mute) = mute {
                    if let Some(mute_role) = g_cfg.mute_role {
                        let _ = new_member.add_role(&ctx, RoleId(mute_role));
                    }
                }
            }
        }
    }

    fn guild_member_removal(
        &self,
        ctx: Context,
        guild_id: GuildId,
        user: User,
        old_member: Option<Member>,
    ) {
        let mut data = ctx.data.write();

        if let Some(pool) = data.get::<DatabaseConnection>() {
            let conn = pool.get().unwrap();

            if let Some(mut config) = server_configs::table
                .filter(server_configs::server_id.eq(*guild_id.as_u64() as i64))
                .first::<ServerConfig>(&*conn)
                .optional()
                .unwrap()
            {
                let g_cfg: GuildConfig = serde_json::from_value(config.config.take()).unwrap();

                let mut user_info = UserInfo {
                    created_at: user.created_at().format("%d.%m.%Y %H:%M:%S").to_string(),
                    created_at_ago: Utc::now()
                        .signed_duration_since(user.created_at())
                        .num_days(),
                    member: None,
                };

                let default = "Unknown".to_string();

                let information_body = format!(
                    "**Left discord:** {} ({} days ago)\n\n**Joined this server:** {} ({} days ago)\n\n**Roles:** {}",
                    user_info.created_at,
                    user_info.created_at_ago,
                    user_info
                        .member
                        .as_ref()
                        .and_then(|m| Some(&m.joined_at))
                        .unwrap_or(&default),
                    user_info
                        .member
                        .as_ref()
                        .and_then(|m| Some(&m.joined_at_ago))
                        .unwrap_or(&default),
                    user_info
                        .member
                        .as_ref()
                        .and_then(|m| Some(m.roles.join(", ")))
                        .unwrap_or_else(|| default.clone()),
                );

                if let Some(userlog_channel) = g_cfg.userlog_channel {
                    let _ = ChannelId(userlog_channel).send_message(&ctx, |m| {
                        m.embed(|e| {
                            e.author(|a| {
                                a.name(&user.name)
                                    .icon_url(&user.static_avatar_url().unwrap_or_default())
                            })
                            .color((0, 120, 220))
                            .description(&information_body)
                            .footer(|f| {
                                f.text(&format!(
                                    "{}#{} | id: {}",
                                    user.name, user.discriminator, &user.id,
                                ))
                            })
                        })
                    });
                }
            }
        }
    }

    fn message(&self, ctx: Context, msg: Message) {
        info!("Message: {:?}", msg);
        if msg.is_private() {
            use crate::schema::tags::dsl::*;
            // check if waiting for labels
            let data = ctx.data.read();
            if let Some(waiter) = data.get::<Waiter>() {
                let mut wait = waiter.lock();
                if let Some(waited_fav_id) = wait.waiting(*msg.author.id.as_u64(), Action::AddTags)
                {
                    let conn = match data.get::<DatabaseConnection>() {
                        Some(v) => v.get().unwrap(),
                        None => {
                            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
                            return;
                        }
                    };

                    // clear old tags for this fav
                    diesel::delete(tags.filter(fav_id.eq(waited_fav_id)))
                        .execute(&conn)
                        .expect("could not delete tags");

                    let received_tags: Vec<NewTag> = msg
                        .content
                        .split(' ')
                        .map(|t| NewTag::new(waited_fav_id, t.to_string()))
                        .collect();
                    crate::models::tag::create_tags(&conn, &received_tags);

                    wait.purge(
                        *msg.author.id.as_u64(),
                        vec![Action::DeleteFav, Action::ReqTags, Action::AddTags],
                    );
                    let _ = msg.reply(&ctx, "added the tags!");
                }
            }
        }
    }

    fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let dispatcher = {
            let mut context = ctx.data.write();
            context
                .get_mut::<crate::DispatcherKey>()
                .expect("No Dispatcher")
                .clone()
        };
        dispatcher
            .write()
            .dispatch_event(&crate::dispatch::DispatchEvent::ReactEvent(
                reaction.message_id,
                reaction.emoji.clone(), //TODO: this can be removed after refactoring old dispatch
                reaction.channel_id,
                reaction.user_id,
            ));

        //TODO: refactor old dispatch style into new one using the dispatcher
        match reaction.emoji {
            ReactionType::Unicode(ref s) if s == "📗" => fav::add(ctx, reaction),
            ReactionType::Unicode(ref s) if s == "🗑" => fav::remove(ctx, reaction),
            ReactionType::Unicode(ref s) if s == "🏷" => fav::add_label(ctx, reaction),
            ReactionType::Unicode(ref s) if s == "👆" => blackjack::hit(ctx, reaction),
            ReactionType::Unicode(ref s) if s == "✋" => blackjack::stay(ctx, reaction),
            ReactionType::Unicode(ref s) if s == "🌀" => blackjack::new_game(ctx, reaction),
            _ => reaction_roles::add_role(ctx, reaction),
        }
    }

    fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        match removed_reaction.emoji {
            ReactionType::Unicode(ref s) if s == "👆" => blackjack::hit(ctx, removed_reaction),
            ReactionType::Unicode(ref s) if s == "✋" => blackjack::stay(ctx, removed_reaction),
            ReactionType::Unicode(ref s) if s == "🌀" => {
                blackjack::new_game(ctx, removed_reaction)
            }
            _ => reaction_roles::remove_role(ctx, removed_reaction),
        }
    }
}
