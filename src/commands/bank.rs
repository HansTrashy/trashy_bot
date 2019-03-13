use crate::interaction::wait::{Action, WaitEvent};
use crate::models::bank::Bank;
use crate::DatabaseConnection;
use crate::Waiter;
use chrono::prelude::*;
use diesel::prelude::*;
use rand::prelude::*;
use serenity::model::{channel::Message, channel::ReactionType, id::ChannelId, id::MessageId};
use crate::schema::banks::dsl::*;

command!(bank(ctx, msg, args) {
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    // check if user already owns a bank
    let results = banks.filter(user_id.eq(*msg.author.id.as_u64() as i64)).load::<Bank>(&*conn.lock()).expect("could not retrieve banks");


    // create bank if not existing
    if results.is_empty() {
        crate::models::bank::create_bank(&*conn.lock(), *msg.author.id.as_u64() as i64, 0, Utc::now().naive_utc());
        let _ = msg.reply("Created bank!");
    } else {
        let _ = msg.reply("You already own a bank");
    }
});

command!(payday(_ctx, msg, args) {
    // check if user has a bank & last payday was over 24h ago
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    // check if user already owns a bank
    let results = banks.filter(user_id.eq(*msg.author.id.as_u64() as i64)).load::<Bank>(&*conn.lock()).expect("could not retrieve banks");

    if results.is_empty() {
        let _ = msg.reply("You do not own a bank, please create one using the bank command");
    } else {
        if Utc::now().naive_utc().signed_duration_since(results[0].last_payday).hours() > 23 {
            let new_amount = results[0].amount + 1000;
            
        }
    }


    // add 1000 to user amount

    if let Err(why) = msg.channel_id.say("Increased your balance by x!") {
        println!("Error sending message: {:?}", why);
    }
});

command!(slot(_ctx, msg, args) {
    // check if user already owns a bank & has enough balance


    // roll

    // add the delta to the user amount

    if let Err(why) = msg.channel_id.say("you get nothing!") {
        println!("Error sending message: {:?}", why);
    }
});
