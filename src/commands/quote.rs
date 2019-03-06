command!(quote(_ctx, msg, args) {
    let quote_msg_id = args.single::<u64>().unwrap();

    let guild = msg.guild_id.ok_or("Failed to get guild_id")?.to_guild_cached();
    let guild = guild.ok_or("failed to get guild")?;

    for (_c_id, channel) in guild.read().channels().unwrap() {
        match channel.message(quote_msg_id) {
            Ok(quoted_msg) => {
                let _ = msg.channel().unwrap().send_message(|m| { m.content(&format!("you quoted: {:?}", quoted_msg.content)) });
                break;
            },
            Err(_e) => (),
        }
    }
});
