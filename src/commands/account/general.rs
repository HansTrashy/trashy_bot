use crate::models::bank::Bank;
use crate::DatabaseConnection;
use chrono::prelude::*;
use log::*;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command]
#[description = "Create an account if you do not already own one"]
#[num_args(0)]
pub fn create(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let mut conn = ctx
        .data
        .read()
        .get::<DatabaseConnection>()
        .map(|v| v.get().expect("pool error"))
        .ok_or("Could not retrieve the database connection!")?;
    // check if user already owns a bank
    if let Ok(bank) = Bank::get(&mut *conn, *msg.author.id.as_u64() as i64) {
        let _ = msg.reply(&ctx, &format!("Your bank balance: {}", bank.amount));
    } else {
        let bank = Bank::create(
            &mut *conn,
            *msg.author.id.as_u64() as i64,
            msg.author.name.to_string(),
            1000,
            Utc::now().naive_utc(),
        );
        debug!("Created bank entry {:?}", bank);
        let _ = msg.reply(&ctx, "Created bank!");
    }
    Ok(())
}

#[command]
#[aliases("paydaddy")]
#[num_args(0)]
pub fn payday(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    // check if user has a bank & last payday was over 24h ago
    let data = ctx.data.read();
    let mut conn = data
        .get::<DatabaseConnection>()
        .map(|v| v.get().expect("pool error"))
        .ok_or("Could not retrieve the database connection!")?;

    if let Ok(bank) = Bank::get(&mut *conn, *msg.author.id.as_u64() as i64) {
        let hours_diff = Utc::now()
            .naive_utc()
            .signed_duration_since(bank.last_payday)
            .num_hours();
        if hours_diff > 23 {
            let updated_amount = bank.amount + 1000;

            Bank::update(
                &mut *conn,
                *msg.author.id.as_u64() as i64,
                updated_amount,
                Utc::now().naive_utc(),
            )?;

            let _ = msg.reply(&ctx, &format!("Your new balance: {}", &updated_amount));
        } else {
            let _ = msg.reply(
                &ctx,
                &format!("Wait {} hours for your next Payday!", (24 - &hours_diff)),
            );
        }
    } else {
        let _ = msg.reply(
            &ctx,
            "You do not own a bank, please create one using the createaccount command",
        );
    }
    Ok(())
}

#[command]
#[description = "Lists the leading players"]
#[num_args(0)]
pub fn leaderboard(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read();
    let mut conn = data
        .get::<DatabaseConnection>()
        .map(|v| v.get().expect("pool error"))
        .ok_or("Could not retrieve the database connection!")?;

    let results = Bank::top10(&mut *conn)?;

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
    let mut conn = data
        .get::<DatabaseConnection>()
        .map(|v| v.get().expect("pool error"))
        .ok_or("Could not retrieve the database connection!")?;
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

    if let Ok(bank) = Bank::get(&mut *conn, *msg.author.id.as_u64() as i64) {
        // check if user has enough balance
        if mentions_count * amount_to_transfer <= bank.amount {
            let updated_amount = bank.amount - mentions_count * amount_to_transfer;

            // remove the needed money
            Bank::update(&mut *conn, bank.user_id, updated_amount, bank.last_payday)?;

            for mention in &msg.mentions {
                if let Ok(bank) = Bank::get(&mut *conn, *mention.id.as_u64() as i64) {
                    let mentioned_user_amount = bank.amount + amount_to_transfer;
                    Bank::update(
                        &mut *conn,
                        bank.user_id,
                        mentioned_user_amount,
                        bank.last_payday,
                    )?;
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
    } else {
        let _ = msg.reply(&ctx, "Du besitzt noch keine Bank!");
    }
    Ok(())
}
