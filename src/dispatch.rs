use std::{collections::HashSet, env, hash::{Hash, Hasher}};
use serenity::{
    prelude::*,
    framework::standard::{
        Args, CommandResult, CommandGroup,
        DispatchError, HelpOptions, help_commands, StandardFramework,
    },
    http::Http,
    model::prelude::*,
};

use hey_listen::sync::{ParallelDispatcher as Dispatcher,
ParallelDispatcherRequest as DispatcherRequest};

#[derive(Clone)]
pub enum DispatchEvent {
    ReactEvent(MessageId, UserId),
}

impl PartialEq for DispatchEvent {
    fn eq(&self, other: &DispatchEvent) -> bool {
        match (self, other) {
            (
                DispatchEvent::ReactEvent(self_msg_id, self_user_id),
                DispatchEvent::ReactEvent(other_msg_id, other_user_id),
            ) => self_msg_id == other_msg_id && self_user_id == other_user_id,
        }
    }
}

impl Eq for DispatchEvent {}

impl Hash for DispatchEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DispatchEvent::ReactEvent(msg_id, user_id) => {
                msg_id.hash(state);
                user_id.hash(state);
            }
        }
    }
}