use log::error;
use serenity::model::channel::Attachment;

command!(quote(_ctx, msg, args) {
    let quote_msg_id = args.single::<u64>().unwrap();

    let guild = msg.guild_id.ok_or("Failed to get guild_id")?.to_guild_cached();
    let guild = guild.ok_or("failed to get guild")?;
    
    let mut found_message = false;
    for (_c_id, channel) in guild.read().channels().unwrap() {
        match channel.message(quote_msg_id) {
            Ok(quoted_msg) => {
                found_message = true;
                if let Some(image) = quoted_msg.attachments.iter().cloned().filter(|a| a.width.is_some()).collect::<Vec<Attachment>>().first() {
                    let _ = msg.channel_id.send_message(|m| {
                    m.embed(|e|
                        e.author(|a| a.name(&quoted_msg.author.name).icon_url(&quoted_msg.author.static_avatar_url().unwrap_or_default()))
                        .color((0,120,220))
                        .description(&quoted_msg.content)
                        .image(&image.url)
                        .footer(|f| f.text(&format!("{} | Zitiert von: {}", &quoted_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"), &msg.author.name)))
                    )});
                } else {
                    let _ = msg.channel_id.send_message(|m| {
                    m.embed(|e|
                        e.author(|a| a.name(&quoted_msg.author.name).icon_url(&quoted_msg.author.static_avatar_url().unwrap_or_default()))
                        .color((0,120,220))
                        .description(&quoted_msg.content)
                        .footer(|f| f.text(&format!("{} | Zitiert von: {}", &quoted_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"), &msg.author.name)))
                    )});
                }
                
                let _ = msg.delete();
                break;
            },
            Err(_e) => error!("failed to get message"),
        }
    }
    if !found_message {
        let _ = msg.reply("Tut mir leid, ich kann diese Nachricht nicht finden.");
    }
});
