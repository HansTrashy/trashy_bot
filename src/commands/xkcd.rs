use serde_derive::Deserialize;
use serenity::{
    framework::standard::{
        Args, CommandResult,
        macros::command,
    },
    model::channel::Message,
};
use serenity::prelude::*;
use log::*;

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
pub fn xkcd(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let xkcd_id = args.single::<u64>()?;

    let comic: Comic = reqwest::get(&format!("https://xkcd.com/{}/info.0.json", xkcd_id))?.json()?;
    let xkcd_link = format!("https://xkcd.com/{}", xkcd_id);

    match msg.channel_id.send_message(&ctx.http, |m| m.embed(|e| 
        e.author(|a| a.name("Xkcd"))
        .title(&comic.title)
        .description(&comic.alt)
        .color((0,120,220))
        .image(&comic.img)
        .url(&xkcd_link)
        .footer(|f| f.text(&xkcd_link)))) {
            Ok(_msg) => Ok(()),
            Err(e) => {
                error!("Failure sending message: {:?}", e);
                Err(e.into())
            }
        }
}