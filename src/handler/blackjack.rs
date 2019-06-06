use crate::BlackjackState;
use serenity::{model::channel::Reaction, prelude::*};

pub fn hit(ctx: Context, reaction: Reaction) {
    let data = ctx.data.read();
    let blackjack_state = data
        .get::<BlackjackState>()
        .expect("No blackjack state available");

    blackjack_state
        .lock()
        .hit(*reaction.user_id.as_u64(), *reaction.message_id.as_u64());
}

pub fn stay(ctx: Context, reaction: Reaction) {
    let data = ctx.data.read();
    let blackjack_state = data
        .get::<BlackjackState>()
        .expect("No blackjack state available");

    blackjack_state
        .lock()
        .stay(*reaction.user_id.as_u64(), *reaction.message_id.as_u64());
}

pub fn new_game(ctx: Context, reaction: Reaction) {
    let data = ctx.data.read();
    let blackjack_state = data
        .get::<BlackjackState>()
        .expect("No blackjack state available");

    blackjack_state
        .lock()
        .new_game(*reaction.user_id.as_u64(), *reaction.message_id.as_u64());
}
