use serenity::model::channel::{
    PermissionOverwrite,
    PermissionOverwriteType,
};
use serenity::model::{ModelError, Permissions};

command!(lockdown(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say("LOCKDOWN, alle in Deckung!") {
        warn!("Error sending message: {:?}", why);
    }
});

command!(unlock(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say("LOCKDOWN beendet, weitermachen!") {
        warn!("Error sending message: {:?}", why);
    }
});