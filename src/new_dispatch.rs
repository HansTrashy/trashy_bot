use serenity::model::prelude::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

type ListenerAction = Box<dyn Fn() + Send + Sync>;

#[derive(Clone, Debug)]
pub enum DispatchEvent {
    ReactEvent(MessageId, ReactionType, ChannelId, UserId),
    OwnerReactEvent(MessageId, ReactionType, ChannelId, UserId),
}

impl PartialEq for DispatchEvent {
    fn eq(&self, other: &DispatchEvent) -> bool {
        match (self, other) {
            (
                DispatchEvent::ReactEvent(s_mid, s_rt, _s_cid, _s_uid),
                DispatchEvent::ReactEvent(o_mid, o_rt, _o_cid, _o_uid),
            ) => s_mid == o_mid && s_rt == o_rt,
            (
                DispatchEvent::OwnerReactEvent(s_mid, s_rt, _s_cid, s_uid),
                DispatchEvent::OwnerReactEvent(o_mid, o_rt, _o_cid, o_uid),
            ) => s_mid == o_mid && s_rt == o_rt && s_uid == o_uid,
            _ => false,
        }
    }
}

impl Eq for DispatchEvent {}

impl Hash for DispatchEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DispatchEvent::ReactEvent(msg_id, reaction_type, _channeld_id, _user_id) => {
                msg_id.hash(state);
                reaction_type.hash(state);
            }
            DispatchEvent::OwnerReactEvent(msg_id, reaction_type, _channeld_id, user_id) => {
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

pub struct Dispatcher<K> {
    listener: HashMap<K, Vec<Listener>>,
}

impl<K> Dispatcher<K>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            listener: HashMap::new(),
        }
    }

    pub fn add_listener(&mut self, id: K, listener: Listener) {
        let entry = self.listener.entry(id).or_default();
        entry.push(listener);
    }

    pub fn dispatch_event(&self, id: &K) {
        if let Some(listener) = self.listener.get(id) {
            for l in listener {
                (l.action)()
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

    #[test]
    fn test_listener() {
        let mut dispatcher = Dispatcher::new();

        dispatcher.add_listener(
            1,
            Listener::new(Duration::from_secs(100), Box::new(|| println!("Test"))),
        );

        dispatcher.dispatch_event(&1);
    }
}
