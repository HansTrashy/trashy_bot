use diesel::pg::PgConnection;
use diesel::prelude::*;
use serenity::{
    client::bridge::gateway::{ShardId, ShardManager},
    framework::standard::{
        help_commands, Args, CommandOptions, DispatchError, HelpBehaviour, StandardFramework,
    },
    model::{
        channel::Message, channel::Reaction, channel::ReactionType, gateway::Ready, Permissions,
    },
    prelude::*,
    utils::{content_safe, ContentSafeOptions},
};

pub struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    fn reaction_add(&self, _ctx: Context, add_reaction: Reaction) {
        match add_reaction.emoji {
            ReactionType::Unicode(ref s) if s == "📗" => {
                // add fav for this user
                let conn = PgConnection::establish("postgres://postgres:root@localhost/trashy_bot")
                    .expect("Error connecting to postgres://postgres:root@localhost/trashy_bot");

                crate::models::favs::create_fav(
                    &conn,
                    *add_reaction
                        .channel()
                        .unwrap()
                        .guild()
                        .unwrap()
                        .read()
                        .guild_id
                        .as_u64() as i64,
                    *add_reaction.channel_id.as_u64() as i64,
                    *add_reaction.message_id.as_u64() as i64,
                    *add_reaction.user_id.as_u64() as i64,
                    *add_reaction.message().unwrap().author.id.as_u64() as i64,
                );

                if let Err(why) = add_reaction.user().unwrap().dm(|m| m.content("Fav saved!")) {
                    println!("Error sending message: {:?}", why);
                }
            }
            _ => (),
        }
    }
}
