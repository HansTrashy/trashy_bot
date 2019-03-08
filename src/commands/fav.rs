use crate::models::favs::Fav;
use crate::schema::favs::dsl::*;
use crate::DatabaseConnection;
use diesel::prelude::*;
use rand::prelude::*;
use serenity::model::{channel::Message, id::ChannelId, id::MessageId};

command!(fav(ctx, msg, _args) {
    let mut rng = rand::thread_rng();
    // select random fav from user
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };

    let results = favs.filter(user_id.eq(*msg.author.id.as_u64() as i64)).load::<Fav>(&*conn.lock()).expect("could not retrieve favs");

    let chosen_fav = results.iter().choose(&mut rng).unwrap();

    let fav_msg = ChannelId(chosen_fav.channel_id as u64).message(chosen_fav.msg_id as u64).unwrap();

    let _ = msg.delete();

    let _ = msg.channel_id.send_message(|m| m.embed(|e| 
        e.author(|a| a.name(&fav_msg.author.name).icon_url(&fav_msg.author.static_avatar_url().unwrap_or_default()))
        .description(&fav_msg.content)
        .color((0,120,220))
        .footer(|f| f.text(&format!("{} | Quoted by: {}", &fav_msg.timestamp.format("%d.%m.%Y, %H:%M:%S"), &msg.author.name)))));
});
