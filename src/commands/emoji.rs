use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command]
#[description = "Post the 'Confused Cat' image"]
async fn katzer(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    std::mem::drop(msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| e.image("https://cdn.discordapp.com/attachments/217015995385118721/632308780477972480/sinnbild.png"))
    }).await);

    Ok(())
}

#[command]
#[description = "Let the bot post an Emoji"]
#[num_args(1)]
#[only_in("guilds")]
async fn emoji(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let emoji_name = args.rest();

    if let Some(guild) = msg.guild(&ctx) {
        for e in guild.emojis.values() {
            if e.name == emoji_name {
                std::mem::drop(
                    msg.channel_id
                        .send_message(&ctx, |m| m.content(format!("{}", e)))
                        .await,
                );
                return Ok(());
            }
        }
    }

    msg.channel_id
        .send_message(&ctx, |m| {
            m.content("Could not find the emoji you are looking for!")
        })
        .await?;

    Ok(())
}
