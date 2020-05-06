use crate::models::bank::Bank;
use crate::DatabasePool;
use chrono::prelude::*;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use tracing::debug;

#[command]
#[description = "Create an account if you do not already own one"]
#[num_args(0)]
pub async fn create(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut conn = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .ok_or("Could not retrieve the database connection!")?
        .get()
        .await?;
    // check if user already owns a bank
    if let Ok(bank) = Bank::get(&mut *conn, *msg.author.id.as_u64() as i64).await {
        let _ = msg
            .reply(ctx, &format!("Your bank balance: {}", bank.amount))
            .await;
    } else {
        let bank = Bank::create(
            &mut *conn,
            *msg.author.id.as_u64() as i64,
            msg.author.name.to_string(),
            1000,
            Utc::now().naive_utc(),
        )
        .await;
        debug!("Created bank entry {:?}", bank);

        let _ = msg.reply(ctx, "Created bank!").await;
    }
    Ok(())
}

#[command]
#[aliases("paydaddy")]
#[num_args(0)]
pub async fn payday(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    // check if user has a bank & last payday was over 24h ago
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .map(|p| p.clone())
        .ok_or("Could not retrieve the database connection!")?;
    let mut conn = pool.get().await?;

    if let Ok(bank) = Bank::get(&mut *conn, *msg.author.id.as_u64() as i64).await {
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
            )
            .await?;

            let _ = msg
                .reply(ctx, &format!("Your new balance: {}", &updated_amount))
                .await;
        } else {
            let _ = msg
                .reply(
                    ctx,
                    &format!("Wait {} hours for your next Payday!", (24 - &hours_diff)),
                )
                .await;
        }
    } else {
        let _ = msg.reply(
            ctx,
            "You do not own a bank, please create one using the createaccount command",
        );
    }
    Ok(())
}

#[command]
#[description = "Lists the leading players"]
#[num_args(0)]
pub async fn leaderboard(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .map(|p| p.clone())
        .ok_or("Could not retrieve the database connection!")?;
    let mut conn = pool.get().await?;

    let results = Bank::top10(&mut *conn).await?;

    let mut rendered_leaderboard = String::from("Top Ten:\n");
    for (i, r) in results.iter().enumerate() {
        rendered_leaderboard.push_str(&format!("\n{} | {} | {}", i + 1, r.amount, r.user_name));
    }

    msg.channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| e.description(&rendered_leaderboard).color((0, 120, 220)))
        })
        .await?;
    Ok(())
}

#[command]
#[description = "Transfers amount x to all listed users"]
#[example = "1000 @user1 @user2"]
pub async fn transfer(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pool = ctx
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .map(|p| p.clone())
        .ok_or("Could not retrieve the database connection!")?;
    let mut conn = pool.get().await?;

    let amount_to_transfer = match args.single::<i64>() {
        Ok(v) if v > 0 => v,
        Ok(_) => {
            // log
            let _ = msg.channel_id.say(&ctx, "Ungültiger Transferbetrag!").await;
            return Ok(());
        }
        Err(_e) => {
            // log
            let _ = msg.channel_id.say(&ctx, "Ungültiger Transferbetrag!").await;
            return Ok(());
        }
    };

    let mentions_count = msg.mentions.len() as i64;

    if let Ok(bank) = Bank::get(&mut *conn, *msg.author.id.as_u64() as i64).await {
        // check if user has enough balance
        if mentions_count * amount_to_transfer <= bank.amount {
            let updated_amount = bank.amount - mentions_count * amount_to_transfer;

            // remove the needed money
            Bank::update(&mut *conn, bank.user_id, updated_amount, bank.last_payday).await?;

            for mention in &msg.mentions {
                if let Ok(bank) = Bank::get(&mut *conn, *mention.id.as_u64() as i64).await {
                    let mentioned_user_amount = bank.amount + amount_to_transfer;
                    Bank::update(
                        &mut *conn,
                        bank.user_id,
                        mentioned_user_amount,
                        bank.last_payday,
                    )
                    .await?;
                }
            }

            let mentioned_user_names: Vec<String> =
                msg.mentions.iter().map(|u| u.name.to_owned()).collect();
            msg.reply(
                ctx,
                &format!(
                    "Transferred: {}, to: {:?}",
                    amount_to_transfer, mentioned_user_names
                ),
            )
            .await?;
        } else {
            msg.reply(ctx, "Du hast nicht genügend credits für den Transfer!")
                .await?;
        }
    } else {
        msg.reply(ctx, "Du besitzt noch keine Bank!").await?;
    }
    Ok(())
}
