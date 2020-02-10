use crate::commands::config::Guild;
use crate::commands::userinfo::UserInfo;
use crate::dispatch::{DispatchEvent, Dispatcher, Listener};
use crate::interaction::wait::Action;
use crate::models::mute::Mute;
use crate::models::server_config::ServerConfig;
use crate::models::tag::Tag;
use crate::DatabaseConnection;
use crate::Waiter;
use chrono::Utc;
use serenity::{
    model::{
        channel::Message, channel::Reaction, channel::ReactionType, gateway::Ready, guild::Member,
        id::ChannelId, id::GuildId, id::RoleId, user::User,
    },
    prelude::*,
};

// mod blackjack;
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
            let mut conn = pool.get().unwrap();

            if let Ok(mut config) = ServerConfig::get(&mut *conn, *guild_id.as_u64() as i64) {
                let g_cfg: Guild = serde_json::from_value(config.config.take()).unwrap();

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
                    "**Joined discord:** {} ({} days ago)\n\n**Has joined this server**",
                    user_info.created_at, user_info.created_at_ago,
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
                            .color((0, 220, 0))
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

                let mute = Mute::get(
                    &mut *conn,
                    *guild_id.as_u64() as i64,
                    *new_member.user.read().id.as_u64() as i64,
                );

                if let Ok(mute) = mute {
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
            let mut conn = pool.get().unwrap();

            if let Ok(mut config) = ServerConfig::get(&mut *conn, *guild_id.as_u64() as i64) {
                let g_cfg: Guild = serde_json::from_value(config.config.take()).unwrap();

                let mut user_info = UserInfo {
                    created_at: user.created_at().format("%d.%m.%Y %H:%M:%S").to_string(),
                    created_at_ago: Utc::now()
                        .signed_duration_since(user.created_at())
                        .num_days(),
                    member: None,
                };

                let default = "Unknown".to_string();

                let information_body = format!(
                    "**Joined discord:** {} ({} days ago)\n\n**Has left the server.**",
                    user_info.created_at, user_info.created_at_ago,
                );

                if let Some(userlog_channel) = g_cfg.userlog_channel {
                    let _ = ChannelId(userlog_channel).send_message(&ctx, |m| {
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
                    });
                }
            }
        }
    }

    fn message(&self, ctx: Context, msg: Message) {
        if msg.is_private() {
            // check if waiting for labels
            let data = ctx.data.read();
            if let Some(waiter) = data.get::<Waiter>() {
                let mut wait = waiter.lock();
                if let Some(waited_fav_id) = wait.waiting(*msg.author.id.as_u64(), Action::AddTags)
                {
                    let mut conn = match data.get::<DatabaseConnection>() {
                        Some(v) => v.get().unwrap(),
                        None => {
                            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
                            return;
                        }
                    };

                    // clear old tags for this fav
                    let _ = Tag::delete(&mut *conn, waited_fav_id);

                    // TODO: make this a single statement
                    msg.content.split(' ').for_each(|tag| {
                        let _ = Tag::create(&mut *conn, waited_fav_id, tag.to_string());
                    });

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
                .get_mut::<crate::TrashyDispatcher>()
                .expect("No new dispatcher")
                .clone()
        };

        dispatcher.lock().dispatch_event(
            &ctx,
            &DispatchEvent::ReactMsg(
                reaction.message_id,
                reaction.emoji.clone(),
                reaction.channel_id,
                reaction.user_id,
            ),
        );

        //TODO: refactor old dispatch style into new one using the dispatcher
        match reaction.emoji {
            ReactionType::Unicode(ref s) if s.starts_with("📗") => fav::add(ctx, reaction),
            ReactionType::Unicode(ref s) if s.starts_with("🗑") => fav::remove(ctx, reaction),
            ReactionType::Unicode(ref s) if s.starts_with("🏷") => fav::add_label(ctx, reaction),
            // ReactionType::Unicode(ref s) if s.starts_with("👆") => blackjack::hit(ctx, reaction),
            // ReactionType::Unicode(ref s) if s.starts_with("✋") => blackjack::stay(ctx, reaction),
            // ReactionType::Unicode(ref s) if s.starts_with("🌀") => {
            //     blackjack::new_game(ctx, reaction)
            // }
            _ => reaction_roles::add_role(ctx, reaction),
        }
    }

    fn reaction_remove(&self, ctx: Context, removed_reaction: Reaction) {
        match removed_reaction.emoji {
            // ReactionType::Unicode(ref s) if s.starts_with("👆") => {
            //     blackjack::hit(ctx, removed_reaction)
            // }
            // ReactionType::Unicode(ref s) if s.starts_with("✋") => {
            //     blackjack::stay(ctx, removed_reaction)
            // }
            // ReactionType::Unicode(ref s) if s.starts_with("🌀") => {
            //     blackjack::new_game(ctx, removed_reaction)
            // }
            _ => reaction_roles::remove_role(ctx, removed_reaction),
        }
    }
}
