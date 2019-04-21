use serde_derive::Deserialize;

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

command!(xkcd(_ctx, msg, args) {
    let xkcd_id = args.single::<u64>().unwrap();

    let comic: Comic = reqwest::get(&format!("https://xkcd.com/{}/info.0.json", xkcd_id)).expect("no answer").json().expect("invalid response");

    let _ = msg.channel_id.send_message(|m| m.embed(|e| 
        e.author(|a| a.name("Xkcd"))
        .title(&comic.title)
        .description(&comic.alt)
        .color((0,120,220))
        .image(&comic.img)));
});
