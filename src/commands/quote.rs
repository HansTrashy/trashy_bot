use log::error;

command!(quote(_ctx, msg, args) {
    let quote_msg_id = args.single::<u64>().unwrap();

    let guild = msg.guild_id.ok_or("Failed to get guild_id")?.to_guild_cached();
    let guild = guild.ok_or("failed to get guild")?;

    for (_c_id, channel) in guild.read().channels().unwrap() {
        match channel.message(quote_msg_id) {
            Ok(quoted_msg) => {
                let _ = msg.channel_id.send_message(|m| {
                    m.embed(|e|
                        e.author(|a| a.name(&quoted_msg.author.name).icon_url(&quoted_msg.author.static_avatar_url().unwrap_or_default()))
                        .color((0,120,220))
                        .description(&quoted_msg.content)
                        .footer(|f| f.text(&format!("{} | Zitiert von: {}", &quoted_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"), &msg.author.name)))
                )});
                break;
            },
            Err(_e) => error!("failed to get message"),
        }
    }
});
