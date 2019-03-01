use crate::models::favs::Fav;
use crate::schema::favs::dsl::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use rand::prelude::*;
use serenity::model::{channel::Message, id::MessageId};

command!(fav(_ctx, msg, _args) {
    let mut rng = rand::thread_rng();
    // select random fav from user
    let conn = PgConnection::establish("postgres://postgres:root@localhost/trashy_bot")
                    .expect("Error connecting to postgres://postgres:root@localhost/trashy_bot");

    let results = favs.load::<Fav>(&conn).expect("could not retrieve favs");

    let chosen_fav = results.iter().choose(&mut rng).unwrap();

    if let Err(why) = msg.channel_id.say(&format!("Fav: {:?}", chosen_fav)) {
        println!("Error sending message: {:?}", why);
    }
});
