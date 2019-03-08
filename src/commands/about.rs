command!(about(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say("Der mülligste aller Bots! : )") {
        println!("Error sending message: {:?}", why);
    }
});
