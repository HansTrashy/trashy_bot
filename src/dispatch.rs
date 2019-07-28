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

#[derive(Clone)]
pub enum DispatchEvent {
    ReactEvent(MessageId, UserId),
}


// implements eq for message id only!
impl PartialEq for DispatchEvent {
    fn eq(&self, other: &DispatchEvent) -> bool {
        match (self, other) {
            (
                DispatchEvent::ReactEvent(self_msg_id, _self_user_id),
                DispatchEvent::ReactEvent(other_msg_id, _other_user_id),
            ) => self_msg_id == other_msg_id
        }
    }
}

impl Eq for DispatchEvent {}

impl Hash for DispatchEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            DispatchEvent::ReactEvent(msg_id, _user_id) => {
                msg_id.hash(state);
            }
        }
    }
}