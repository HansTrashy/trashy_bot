use crate::models::reminder::Reminder;
use crate::util;
use crate::util::get_client;
use chrono::Utc;
use serenity::utils::MessageBuilder;
use serenity::utils::{content_safe, ContentSafeOptions};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};
use tokio::time::delay_for;

#[command]
#[description = "Reminds you after the given time with the given text. Allows (w, d, h, m, s)"]
#[example("15m Pizza ist fertig!")]
#[usage("*duration* *message*")]
#[min_args(1)]
pub async fn remindme(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let duration = util::parse_duration(&args.single::<String>()?);
    let pool = get_client(&ctx).await?;

    match duration {
        None => {
            let _ = msg
                .reply(ctx, "Unknown time unit. Allowed units are: s,m,h,d,w")
                .await;
        }
        Some(duration) => {
            let defaults = ContentSafeOptions::default();
            let message = content_safe(&ctx, args.rest().to_string(), &defaults).await;

            Reminder::create(
                &pool,
                *msg.channel_id.as_u64() as i64,
                *msg.id.as_u64() as i64,
                *msg.author.id.as_u64() as i64,
                Utc::now() + duration,
                &message,
            )
            .await?;

            let _ = msg
                .react(ctx, ReactionType::Unicode("✅".to_string()))
                .await;

            delay_for(duration.to_std()?).await;

            let _ = Reminder::delete(&pool, *msg.id.as_u64() as i64).await;

            let _ = msg
                .channel_id
                .send_message(ctx, |m| {
                    m.content(
                        MessageBuilder::new()
                            .push("Hey, ")
                            .mention(&msg.author.id)
                            .push("! You wanted me to remind you that: ")
                            .push(message)
                            .build(),
                    )
                })
                .await;
        }
    }
    Ok(())
}
