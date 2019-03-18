command!(about(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say("Der mülligste aller Bots!\nGit: https://github.com/HansTrashy/trashy_bot") {
        println!("Error sending message: {:?}", why);
    }
});
