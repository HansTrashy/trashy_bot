use log::*;
use serde::Deserialize;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command]
#[description = "Let spongebob say something"]
#[aliases("sponge")]
fn spongebob(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let spongify_this: String = args
        .rest()
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if (i % 2) == 0 {
                c.to_uppercase().to_string()
            } else {
                c.to_string()
            }
        })
        .collect();

    let _ = msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.author(|a| {
                a.name("Spongebob")
                    .icon_url("https://cdn.discordapp.com/emojis/598837367343808532.png?v=1")
            })
            .description(&spongify_this)
            .footer(|f| f.text(&format!("Spongified by: {}", &msg.author.name)))
            .color((0, 120, 220))
        })
    });

    let _ = msg.delete(ctx);

    Ok(())
}
