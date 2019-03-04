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

mod fav;

pub struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }

    fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        match add_reaction.emoji {
            ReactionType::Unicode(ref s) if s == "📗" => {
                // add fav for this user
                fav::fav(ctx, add_reaction);
            }
            _ => (),
        }
    }
}
