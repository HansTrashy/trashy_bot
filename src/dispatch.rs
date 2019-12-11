use serenity::{
    framework::standard::{
        help_commands, Args, CommandGroup, CommandResult, DispatchError, HelpOptions,
        StandardFramework,
    },
    http::Http,
    model::prelude::*,
    prelude::*,
};
use std::{
    collections::HashSet,
    env,
    hash::{Hash, Hasher},
};

#[derive(Clone)]
pub enum DispatchEvent {
    ReactEvent(MessageId, ReactionType, ChannelId, UserId),
}

// implements eq for message id only!
impl PartialEq for DispatchEvent {
    fn eq(&self, other: &DispatchEvent) -> bool {
        match (self, other) {
            (
                DispatchEvent::ReactEvent(
                    self_msg_id,
                    self_reaction_type,
                    _self_channel_id,
                    _self_user_id,
                ),
                DispatchEvent::ReactEvent(
                    other_msg_id,
                    other_reaction_type,
                    _other_channel_id,
                    _other_user_id,
                ),
            ) => self_msg_id == other_msg_id && self_reaction_type == self_reaction_type,
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
        }
    }
}
