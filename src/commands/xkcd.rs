use serde::Deserialize;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use tracing::error;

#[derive(Debug, Deserialize)]
pub struct Comic {
    month: String,
    num: u64,
    link: String,
    year: String,
    news: String,
    safe_title: String,
    transcript: String,
    alt: String,
    img: String,
    title: String,
    day: String,
}

#[command]
#[description = "Post the xkcd comic specified"]
#[example = "547"]
#[num_args(1)]
pub async fn xkcd(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let xkcd_id = args.single::<u64>()?;

    let comic: Comic = reqwest::get(&format!("https://xkcd.com/{}/info.0.json", xkcd_id))
        .await?
        .json()
        .await?;
    let xkcd_link = format!("https://xkcd.com/{}", xkcd_id);

    match msg
        .channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.author(|a| a.name("Xkcd"))
                    .title(&comic.title)
                    .description(&comic.alt)
                    .color((0, 120, 220))
                    .image(&comic.img)
                    .url(&xkcd_link)
                    .footer(|f| f.text(&xkcd_link))
            })
        })
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failure sending message: {:?}", e);
            Err(e.into())
        }
    }
}
