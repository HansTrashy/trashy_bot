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

    let results = favs.load::<Fav>(&*conn.lock()).expect("could not retrieve favs");

    let chosen_fav = results.iter().choose(&mut rng).unwrap();

    let fav_msg = ChannelId(chosen_fav.channel_id as u64).message(chosen_fav.msg_id as u64).unwrap();

    if let Err(why) = msg.channel_id.say(&format!("Fav: {}", fav_msg.content)) {
        println!("Error sending message: {:?}", why);
    }
});
