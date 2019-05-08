use log::*;
use serenity::model::channel::{Channel, PermissionOverwrite, PermissionOverwriteType};
use serenity::model::id::RoleId;
use serenity::model::Permissions;
use crate::LockdownState;

command!(lockdown(ctx, msg, _args) {
    let data = ctx.data.lock();
    let lockdown_state = match data.get::<LockdownState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the lockdown state!");
            return Ok(());
        }
    };

    // check if overwrite already active
    if lockdown_state.lock().is_active(*msg.channel_id.as_u64()) {
        return Ok(());
    }

    let channel = msg.channel_id.to_channel().expect("Could not request channel via REST.");
    // get old everyone role overwrites
    if let Channel::Guild(channel) = channel {
        let channel = channel.read();
        let guild_id = *msg.guild_id.unwrap().as_u64();

        let old_overwrite = channel.permission_overwrites.iter().cloned().filter(|ow| ow.kind == PermissionOverwriteType::Role(RoleId::from(guild_id))).collect::<Vec<PermissionOverwrite>>().first().unwrap().to_owned();

        lockdown_state.lock().insert(*msg.channel_id.as_u64(), channel.permission_overwrites.clone());

        // set everyone role to not allow sending & reacting
        let allow = Permissions::empty();
        let deny = old_overwrite.deny | Permissions::SEND_MESSAGES | Permissions::ADD_REACTIONS;
        let overwrite = PermissionOverwrite {
            allow: allow,
            deny: deny,
            kind: PermissionOverwriteType::Role(RoleId::from(guild_id)),
        };


    
        channel.create_permission(&overwrite)?;
    }

    if let Err(why) = msg.channel_id.say("LOCKDOWN, alle in Deckung!") {
        warn!("Error sending message: {:?}", why);
    }
});

command!(unlock(_ctx, msg, _args) {

    if let Err(why) = msg.channel_id.say("LOCKDOWN beendet, weitermachen!") {
        warn!("Error sending message: {:?}", why);
    }
});
