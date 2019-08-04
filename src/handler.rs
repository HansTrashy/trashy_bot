use crate::interaction::wait::Action;
use crate::models::tag::NewTag;
use crate::DatabaseConnection;
use crate::Waiter;
use diesel::prelude::*;
use log::*;
use serenity::{
    model::{
        channel::Message, channel::Reaction, channel::ReactionType, gateway::Ready, id::GuildId,
        user::User, guild::Member, id::ChannelId,
    },
    prelude::*,
};
use crate::schema::server_configs;
use crate::schema::mutes;
use crate::models::server_config::{ServerConfig, NewServerConfig};
use crate::models::mute::Mute;
use crate::commands::config::GuildConfig;

mod blackjack;
mod fav;
mod reaction_roles;

pub struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    fn guild_member_addition(&self, ctx: Context, guild_id: GuildId, new_member: Member) {
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

                if let Some(userlog_channel) = g_cfg.userlog_channel {
                    let _ = ChannelId(userlog_channel).send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.color((0, 120, 220)).description(format!(
                                "user {} joined!",
                                new_member.user.read().name
                            ))
                        })
                    });
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

                if let Some(userlog_channel) = g_cfg.userlog_channel {
                    let _ = ChannelId(userlog_channel).send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.color((0, 120, 220))
                                .description(format!("user {} left!", user.name))
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
