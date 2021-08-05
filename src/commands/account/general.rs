use crate::models::bank::Bank;
use crate::util::get_client;
use chrono::prelude::*;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use tracing::debug;

#[command]
#[description = "Create a slot account"]
#[num_args(0)]
pub async fn create(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let pool = get_client(ctx).await?;
    // check if user already owns a bank
    if let Ok(bank) = Bank::get(&pool, *msg.author.id.as_u64() as i64).await {
        std::mem::drop(
            msg.reply(ctx, &format!("Your bank balance: {}", bank.amount))
                .await,
        );
    } else {
        let bank = Bank::create(
            &pool,
            *msg.author.id.as_u64() as i64,
            msg.author.name.to_string(),
            1000,
            Utc::now().naive_utc(),
        )
        .await;
        debug!("Created bank entry {:?}", bank);

        std::mem::drop(msg.reply(ctx, "Created bank!").await);
    }
    Ok(())
}

#[command]
#[description = "Receive your daily allowance credits"]
#[aliases("paydaddy")]
#[num_args(0)]
pub async fn payday(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let pool = get_client(ctx).await?;
    // check if user has a bank & last payday was over 24h ago

    if let Ok(bank) = Bank::get(&pool, *msg.author.id.as_u64() as i64).await {
        let hours_diff = Utc::now()
            .naive_utc()
            .signed_duration_since(bank.last_payday)
            .num_hours();
        if hours_diff > 23 {
            let updated_amount = if bank.amount < 0 {
                1000
            } else {
                bank.amount + 1000
            };

            Bank::update(
                &pool,
                *msg.author.id.as_u64() as i64,
                updated_amount,
                Utc::now().naive_utc(),
            )
            .await?;

            std::mem::drop(
                msg.reply(ctx, &format!("Your new balance: {}", &updated_amount))
                    .await,
            );
        } else {
            std::mem::drop(
                msg.reply(
                    ctx,
                    &format!("Wait {} hours for your next Payday!", (24 - &hours_diff)),
                )
                .await,
            );
        }
    } else {
        std::mem::drop(
            msg.reply(ctx, "Create your own bank first by running 'acc create'")
                .await,
        );
    }
    Ok(())
}

#[command]
#[description = "List the leading players"]
#[num_args(0)]
pub async fn leaderboard(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let pool = get_client(ctx).await?;
    let results = Bank::top10(&pool).await?;

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
#[description = "Transfer credits from your bank to a list of other users"]
#[usage = "*amount* *from_user_mention* *to_user_mention*"]
#[example = "1000 @HansTrashy @ApoY2k"]
pub async fn transfer(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pool = get_client(ctx).await?;
    let amount_to_transfer = match args.single::<i64>() {
        Ok(v) if v > 0 => v,
        Ok(_) => {
            // log
            std::mem::drop(msg.channel_id.say(&ctx, "Invalid credit amount!").await);
            return Ok(());
        }
        Err(_e) => {
            // log
            std::mem::drop(msg.channel_id.say(&ctx, "Invalid credit amount!").await);
            return Ok(());
        }
    };

    let mentions_count = msg.mentions.len() as i64;

    if let Ok(bank) = Bank::get(&pool, *msg.author.id.as_u64() as i64).await {
        // check if user has enough balance
        if mentions_count * amount_to_transfer <= bank.amount {
            let updated_amount = bank.amount - mentions_count * amount_to_transfer;

            // remove the needed money
            Bank::update(&pool, bank.user_id, updated_amount, bank.last_payday).await?;

            for mention in &msg.mentions {
                if let Ok(bank) = Bank::get(&pool, *mention.id.as_u64() as i64).await {
                    let mentioned_user_amount = bank.amount + amount_to_transfer;
                    Bank::update(&pool, bank.user_id, mentioned_user_amount, bank.last_payday)
                        .await?;
                }
            }

            let mentioned_user_names: Vec<String> =
                msg.mentions.iter().map(|u| u.name.clone()).collect();
            msg.reply(
                ctx,
                &format!(
                    "Transferred: {}, to: {:?}",
                    amount_to_transfer, mentioned_user_names
                ),
            )
            .await?;
        } else {
            msg.reply(
                ctx,
                "You cannot transfer more credits than you have in your bank!",
            )
            .await?;
        }
    } else {
        msg.reply(ctx, "Create your own bank first by running 'acc create'")
            .await?;
    }
    Ok(())
}
