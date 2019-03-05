command!(ban(_ctx, msg, _args) {
    let users_to_ban = &msg.mentions;

    let guild = msg.guild_id.ok_or("Failed to get guild_id")?.to_guild_cached();
    let guild = guild.ok_or("failed to get guild")?;

    for user in users_to_ban {
        guild.read().member(user.id)?.ban(&0)?
    }

    if let Err(why) = msg.channel_id.say("Banned users user(s)!") {
        println!("Error sending message: {:?}", why);
    }
});
