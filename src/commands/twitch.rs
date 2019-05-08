use log::*;
use std::io::{self, Write};
use serenity::model::channel::{Channel, PermissionOverwrite, PermissionOverwriteType};
use serenity::model::id::RoleId;
use serenity::model::{ModelError, Permissions};

command!(twitch(_ctx, msg, args) {
        info!("In Twitch Command");
        msg.channel_id.say("Twitch!");
    });