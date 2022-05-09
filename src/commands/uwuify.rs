use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use uwuifier::uwuify_str_sse;

#[command]
#[description = "uwuify text"]
#[aliases("uwu")]
async fn uwuify(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let uwu_this: String = uwuify_str_sse(args.rest());

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.author(|a| a.name("UwU"))
                    .description(&uwu_this)
                    .footer(|f| f.text(&format!("uwuified by: {}", &msg.author.name)))
                    .color((239, 66, 245))
            })
        })
        .await?;

    msg.delete(ctx).await?;

    Ok(())
}
