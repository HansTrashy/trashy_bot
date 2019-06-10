use crate::models::bank::Bank;
use crate::schema::banks::dsl::*;
use crate::DatabaseConnection;
use chrono::prelude::*;
use diesel::prelude::*;
use serenity::{
    framework::standard::{
        Args, CommandResult,
        macros::command,
    },
    model::channel::Message,
};
use serenity::prelude::*;
use log::*;

#[command]
#[description = "Create an account if you do not already own one"]
#[num_args(0)]
pub fn createaccount(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    // check if user already owns a bank
    let results = banks
        .filter(user_id.eq(*msg.author.id.as_u64() as i64))
        .load::<Bank>(&conn)
        .expect("could not retrieve banks");

    // create bank if not existing
    if results.is_empty() {
        crate::models::bank::create_bank(
            &conn,
            *msg.author.id.as_u64() as i64,
            msg.author.name.to_string(),
            1000,
            Utc::now().naive_utc(),
        );
        let _ = msg.reply(ctx, "Created bank!");
    } else {
        let _ = msg.reply(ctx, &format!("Your bank balance: {}", results[0].amount));
    }
    Ok(())
}

#[command]
#[aliases("paydaddy")]
#[num_args(0)]
pub fn payday(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    // check if user has a bank & last payday was over 24h ago
    let data = ctx.data.read();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    // check if user already owns a bank
    let results = banks
        .filter(user_id.eq(*msg.author.id.as_u64() as i64))
        .load::<Bank>(&conn)
        .expect("could not retrieve banks");

    if results.is_empty() {
        let _ = msg.reply(
            &ctx,
            "You do not own a bank, please create one using the createaccount command",
        );
    } else {
        let hours_diff = Utc::now()
            .naive_utc()
            .signed_duration_since(results[0].last_payday)
            .num_hours();
        if hours_diff > 23 {
            let updated_amount = results[0].amount + 1000;

            diesel::update(banks.filter(user_id.eq(*msg.author.id.as_u64() as i64)))
                .set((
                    amount.eq(updated_amount),
                    last_payday.eq(Utc::now().naive_utc()),
                ))
                .execute(&conn)
                .expect("failed update bank");
            let _ = msg.reply(&ctx, &format!("Your new balance: {}", &updated_amount));
        } else {
            let _ = msg.reply(
                &ctx,
                &format!("Wait {} hours for your next Payday!", (24 - &hours_diff)),
            );
        }
    }
    Ok(())
}

#[command]
#[description = "Lists the leading players"]
#[num_args(0)]
pub fn leaderboard(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg
                .channel_id
                .say(&ctx, "Datenbankfehler, bitte informiere einen Moderator!");
            return Ok(());
        }
    };
    // get top 10 on leaderboard
    let results = banks
        .order(amount.desc())
        .limit(10)
        .load::<Bank>(&conn)
        .expect("could not retrieve banks");

    let mut rendered_leaderboard = String::from("Top Ten:\n");
    for (i, r) in results.iter().enumerate() {
        rendered_leaderboard.push_str(&format!("\n{} | {} | {}", i + 1, r.amount, r.user_name));
    }

    let _ = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| e.description(&rendered_leaderboard).color((0, 120, 220)))
    });
    Ok(())
}

#[command]
#[description = "Transfers amount x to all listed users"]
#[example = "1000 @user1 @user2"]
pub fn transfer(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg
                .channel_id
                .say(&ctx, "Datenbankfehler, bitte informiere einen Moderator!");
            return Ok(());
        }
    };
    let amount_to_transfer = match args.single::<i64>() {
        Ok(v) if v > 0 => v,
        Ok(_) => {
            // log
            let _ = msg.channel_id.say(&ctx, "Ungültiger Transferbetrag!");
            return Ok(());
        }
        Err(_e) => {
            // log
            let _ = msg.channel_id.say(&ctx, "Ungültiger Transferbetrag!");
            return Ok(());
        }
    };

    let mentions_count = msg.mentions.len() as i64;

    // get user entry
    let results = banks
        .filter(user_id.eq(*msg.author.id.as_u64() as i64))
        .load::<Bank>(&conn)
        .expect("could not retrieve banks");

    // check if user has bank
    if results.is_empty() {
        let _ = msg.reply(&ctx, "Du besitzt noch keine Bank!");
    } else {
        // check if user has enough balance
        if mentions_count * amount_to_transfer <= results[0].amount {
            let updated_amount = results[0].amount - mentions_count * amount_to_transfer;

            // remove the needed money
            diesel::update(banks.filter(id.eq(results[0].id)))
                .set(amount.eq(updated_amount))
                .execute(&conn)
                .expect("failed update bank");

            for mention in &msg.mentions {
                let mentioned_users = banks
                    .filter(user_id.eq(*mention.id.as_u64() as i64))
                    .load::<Bank>(&conn)
                    .expect("could not retrieve banks");
                if !mentioned_users.is_empty() {
                    let mentioned_user_amount = mentioned_users[0].amount + amount_to_transfer;
                    diesel::update(banks.filter(id.eq(mentioned_users[0].id)))
                        .set(amount.eq(mentioned_user_amount))
                        .execute(&conn)
                        .expect("failed update bank");
                }
            }

            let mentioned_user_names: Vec<String> =
                msg.mentions.iter().map(|u| u.name.to_owned()).collect();
            let _ = msg.reply(
                &ctx,
                &format!(
                    "Transferred: {}, to: {:?}",
                    amount_to_transfer, mentioned_user_names
                ),
            );
        } else {
            let _ = msg.reply(&ctx, "Du hast nicht genügend credits für den Transfer!");
        }
    }
    Ok(())
}
