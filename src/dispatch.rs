use serenity::model::prelude::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};
use serenity::client::Context;

type ListenerAction = Box<dyn Fn(&Context, &DispatchEvent) + Send + Sync>;

#[derive(Clone, Debug)]
pub enum DispatchEvent {
    React(MessageId, ReactionType, ChannelId, UserId),
    ReactMsg(MessageId, ReactionType, ChannelId, UserId),
    ReactMsgOwner(MessageId, ReactionType, ChannelId, UserId),
}

impl PartialEq for DispatchEvent {
    fn eq(&self, other: &DispatchEvent) -> bool {
        match (self, other) {
            (
                DispatchEvent::React(_s_mid, s_rt, _s_cid, _s_uid),
                DispatchEvent::React(_o_mid, o_rt, _o_cid, _o_uid),
            ) => s_rt == o_rt,
            (
                DispatchEvent::ReactMsg(s_mid, s_rt, _s_cid, _s_uid),
                DispatchEvent::ReactMsg(o_mid, o_rt, _o_cid, _o_uid),
            ) => s_mid == o_mid && s_rt == o_rt,
            (
                DispatchEvent::ReactMsgOwner(s_mid, s_rt, _s_cid, s_uid),
                DispatchEvent::ReactMsgOwner(o_mid, o_rt, _o_cid, o_uid),
            ) => s_mid == o_mid && s_rt == o_rt && s_uid == o_uid,
            _ => false,
        }
    }
}

impl Eq for DispatchEvent {}

impl Hash for DispatchEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DispatchEvent::React(_msg_id, reaction_type, _channeld_id, _user_id) => {
                reaction_type.hash(state);
            }
            DispatchEvent::ReactMsg(msg_id, reaction_type, _channeld_id, _user_id) => {
                msg_id.hash(state);
                reaction_type.hash(state);
            }
            DispatchEvent::ReactMsgOwner(msg_id, reaction_type, _channeld_id, user_id) => {
                msg_id.hash(state);
                reaction_type.hash(state);
                user_id.hash(state);
            }
        }
    }
}

pub struct Listener {
    action: ListenerAction,
    expiration: Instant,
}

impl Listener {
    pub fn new(duration: Duration, action: ListenerAction) -> Self {
        Self {
            action,
            expiration: Instant::now() + duration,
        }
    }
}

pub struct Dispatcher {
    listener: HashMap<DispatchEvent, Vec<Listener>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            listener: HashMap::new(),
        }
    }

    pub fn add_listener(&mut self, id: DispatchEvent, listener: Listener) {
        let entry = self.listener.entry(id).or_default();
        entry.push(listener);
    }

    pub fn dispatch_event(&self, ctx: &Context, id: &DispatchEvent) {
        if let Some(listener) = self.listener.get(id) {
            for l in listener {
                (l.action)(ctx, id)
            }
        }
    }

    pub fn check_expiration(&mut self) {
        for (_, listener) in self.listener.iter_mut() {
            listener.retain(|l| Instant::now() < l.expiration);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_listener() {
    //     let mut dispatcher = Dispatcher::new();

    //     dispatcher.add_listener(
    //         DispatchEvent::React(
    //             MessageId::default(),
    //             ReactionType::from("📗"),
    //             ChannelId::default(),
    //             UserId::default(),
    //         ),
    //         Listener::new(Duration::from_secs(100), Box::new(|_| println!("Test"))),
    //     );

    //     dispatcher.dispatch_event(Context::default(), &DispatchEvent::React(
    //         MessageId::default(),
    //         ReactionType::from("📗"),
    //         ChannelId::default(),
    //         UserId::default(),
    //     ));
    // }
}
