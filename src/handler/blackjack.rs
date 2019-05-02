use crate::interaction::wait::Action;
use crate::interaction::wait::WaitEvent;
use crate::BlackjackState;
use crate::DatabaseConnection;
use crate::Waiter;
use chrono::prelude::*;
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

pub fn hit(ctx: Context, reaction: Reaction) {
    let data = ctx.data.lock();
    let blackjack_state = data
        .get::<BlackjackState>()
        .expect("No blackjack state available");

    blackjack_state.lock().hit(*reaction.user_id.as_u64());
}

pub fn stay(ctx: Context, reaction: Reaction) {
    let data = ctx.data.lock();
    let blackjack_state = data
        .get::<BlackjackState>()
        .expect("No blackjack state available");

    blackjack_state.lock().stay(*reaction.user_id.as_u64());
}

pub fn new_game(ctx: Context, reaction: Reaction) {
    let data = ctx.data.lock();
    let blackjack_state = data
        .get::<BlackjackState>()
        .expect("No blackjack state available");

    blackjack_state.lock().new_game(*reaction.user_id.as_u64());
}
