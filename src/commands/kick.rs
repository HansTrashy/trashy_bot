command!(kick(_ctx, msg, _args) {
    let users_to_kick = &msg.mentions;

    let guild = msg.guild_id.ok_or("Failed to get guild_id")?.to_guild_cached();
    let guild = guild.ok_or("failed to get guild")?;

    for user in users_to_kick {
        guild.read().member(user.id)?.kick()?
    }

    if let Err(why) = msg.channel_id.say("Kicked user(s)!") {
        println!("Error sending message: {:?}", why);
    }
});
