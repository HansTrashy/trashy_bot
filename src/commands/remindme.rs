use crate::util;
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
pub async fn remindme(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let duration = util::parse_duration(&args.single::<String>()?);

    match duration {
        None => {
            let _ = msg
                .reply(&ctx, "Unknown time unit. Allowed units are: s,m,h,d,w")
                .await;
        }
        Some(duration) => {
            let defaults = ContentSafeOptions::default();
            let message = content_safe(&ctx, args.rest().to_string(), &defaults).await;

            msg.react(&ctx, ReactionType::Unicode("✅".to_string()))
                .await?;

            delay_for(duration.to_std()?).await;

            let _ = msg.reply(ctx, message).await;
        }
    }
    Ok(())
}
