use futures::future::BoxFuture;
use serenity::client::Context;
use serenity::model::prelude::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};
use tracing::debug;

type Action = Box<dyn Fn(Context, Event) -> BoxFuture<'static, ()> + Sync + Send>;

#[derive(Clone, Debug)]
pub enum Event {
    React(MessageId, ReactionType, ChannelId, UserId), //TODO: check usefulness
    ReactMsg(MessageId, ReactionType, ChannelId, UserId),
    ReactMsgOwner(MessageId, ReactionType, ChannelId, UserId), //TODO: check usefulness
}

impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        match (self, other) {
            (
                Event::React(_s_mid, s_rt, _s_cid, _s_uid),
                Event::React(_o_mid, o_rt, _o_cid, _o_uid),
            ) => s_rt == o_rt,
            (
                Event::ReactMsg(s_mid, s_rt, _s_cid, _s_uid),
                Event::ReactMsg(o_mid, o_rt, _o_cid, _o_uid),
            ) => s_mid == o_mid && s_rt == o_rt,
            (
                Event::ReactMsgOwner(s_mid, s_rt, _s_cid, s_uid),
                Event::ReactMsgOwner(o_mid, o_rt, _o_cid, o_uid),
            ) => s_mid == o_mid && s_rt == o_rt && s_uid == o_uid,
            _ => false,
        }
    }
}

impl Eq for Event {}

impl Hash for Event {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Event::React(_msg_id, reaction_type, _channeld_id, _user_id) => {
                reaction_type.hash(state);
            }
            Event::ReactMsg(msg_id, reaction_type, _channeld_id, _user_id) => {
                msg_id.hash(state);
                reaction_type.hash(state);
            }
            Event::ReactMsgOwner(msg_id, reaction_type, _channeld_id, user_id) => {
                msg_id.hash(state);
                reaction_type.hash(state);
                user_id.hash(state);
            }
        }
    }
}

pub struct Listener {
    action: Action,
    expiration: Instant,
}

impl Listener {
    pub fn new(duration: Duration, action: Action) -> Self {
        Self {
            action,
            expiration: Instant::now() + duration,
        }
    }
}

pub struct Dispatcher {
    listener: HashMap<Event, Vec<Listener>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            listener: HashMap::new(),
        }
    }

    pub fn add_listener(&mut self, id: Event, listener: Listener) {
        let entry = self.listener.entry(id).or_default();
        entry.push(listener);
    }

    pub async fn dispatch_event(&self, ctx: Context, id: Event) {
        debug!(event = ?id, "Dispatching event");
        if let Some(listener) = self.listener.get(&id) {
            let mut futures = Vec::new();
            for l in listener {
                futures.push((l.action)(ctx.clone(), id.clone()));
            }
            futures::future::join_all(futures).await;
        }
    }

    pub fn check_expiration(&mut self) {
        for listener in self.listener.values_mut() {
            listener.retain(|l| Instant::now() < l.expiration);
        }
    }
}
