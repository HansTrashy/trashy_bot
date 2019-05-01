use crate::interaction::wait::{Action, WaitEvent};
use crate::models::bank::Bank;
use crate::schema::banks::dsl::*;
use crate::DatabaseConnection;
use crate::Waiter;
use chrono::prelude::*;
use diesel::prelude::*;
use rand::prelude::*;
use serenity::model::{channel::Message, channel::ReactionType, id::ChannelId, id::MessageId};
use serenity::utils::{content_safe, ContentSafeOptions};
use std::fmt;

command!(play(ctx, msg, args) {

});
