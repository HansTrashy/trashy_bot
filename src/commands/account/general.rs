use crate::models::bank::Bank;
use crate::schema::banks::dsl::*;
use crate::DatabaseConnection;
use chrono::prelude::*;
use diesel::prelude::*;

command!(createaccount(ctx, msg, _args) {
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    dbg!(*msg.author.id.as_u64());
    // check if user already owns a bank
    let results = banks.filter(user_id.eq(*msg.author.id.as_u64() as i64)).load::<Bank>(&*conn.lock()).expect("could not retrieve banks");

    // create bank if not existing
    if results.is_empty() {
        crate::models::bank::create_bank(&*conn.lock(), *msg.author.id.as_u64() as i64, msg.author.name.to_string(), 1000, Utc::now().naive_utc());
        let _ = msg.reply("Created bank!");
    } else {
        let _ = msg.reply(&format!("Your bank balance: {}", results[0].amount));
    }
});

command!(payday(ctx, msg, _args) {
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
        let _ = msg.reply("You do not own a bank, please create one using the createaccount command");
    } else {
        let hours_diff = Utc::now().naive_utc().signed_duration_since(results[0].last_payday).num_hours();
        if  hours_diff > 23 {
            let updated_amount =results[0].amount + 1000;

            diesel::update(banks.filter(user_id.eq(*msg.author.id.as_u64() as i64)))
                .set((amount.eq(updated_amount), last_payday.eq(Utc::now().naive_utc())))
                .execute(&*conn.lock())
                .expect("failed update bank");
            let _ = msg.reply(&format!("Your new balance: {}", &updated_amount));
        } else {
            let _ = msg.reply(&format!("Wait {} hours for your next Payday!", (24 - &hours_diff)));
        }
    }
});

command!(leaderboard(ctx, msg, _args) {
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.channel_id.say("Datenbankfehler, bitte informiere einen Moderator!");
            return Ok(());
        }
    };
    // get top 10 on leaderboard
    let results = banks.order(amount.desc()).limit(10).load::<Bank>(&*conn.lock()).expect("could not retrieve banks");

    let mut rendered_leaderboard = String::from("Top Ten:\n");
    for (i, r) in results.iter().enumerate() {
        rendered_leaderboard.push_str(&format!("\n{} | {} | {}", i + 1, r.amount, r.user_name));
    }

    let _ = msg.channel_id.send_message(|m| m.embed(|e| 
                e.description(&rendered_leaderboard)
                .color((0,120,220))));
});

command!(transfer(ctx, msg, args) {
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.channel_id.say("Datenbankfehler, bitte informiere einen Moderator!");
            return Ok(());
        }
    };
    let amount_to_transfer = match args.single::<i64>() {
        Ok(v) if v > 0 => v,
        Ok(_) => {
            // log
            let _ = msg.channel_id.say("Ungültiger Transferbetrag!");
            return Ok(());
        }
        Err(_e) => {
            // log
            let _ = msg.channel_id.say("Ungültiger Transferbetrag!");
            return Ok(());
        }
    };

    let mentions_count = msg.mentions.len() as i64;


    // get user entry
    let results = banks.filter(user_id.eq(*msg.author.id.as_u64() as i64)).load::<Bank>(&*conn.lock()).expect("could not retrieve banks");

    // check if user has bank
    if !results.is_empty() {

        // check if user has enough balance
        if mentions_count * amount_to_transfer <= results[0].amount {

            let updated_amount = results[0].amount - mentions_count * amount_to_transfer;

            // remove the needed money
            diesel::update(banks.filter(id.eq(results[0].id))).set(amount.eq(updated_amount)).execute(&*conn.lock()).expect("failed update bank");

            for mention in &msg.mentions {
                let mentioned_users = banks.filter(user_id.eq(*mention.id.as_u64() as i64)).load::<Bank>(&*conn.lock()).expect("could not retrieve banks");
                if !mentioned_users.is_empty() {
                    let mentioned_user_amount = mentioned_users[0].amount + amount_to_transfer;
                    diesel::update(banks.filter(id.eq(mentioned_users[0].id))).set(amount.eq(mentioned_user_amount)).execute(&*conn.lock()).expect("failed update bank");
                }
            }

            let mentioned_user_names: Vec<String> = msg.mentions.iter().map(|u| u.name.to_owned()).collect();
            let _ = msg.reply(&format!("Transferred: {}, to: {:?}", amount_to_transfer, mentioned_user_names));

        } else {
            let _ = msg.reply("Du hast nicht genügend credits für den Transfer!");
        }
    } else {
        let _ = msg.reply("Du besitzt noch keine Bank!");
    }
});