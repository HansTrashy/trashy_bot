use rand::prelude::*;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use tracing::error;

#[command]
#[description = "Roll x dice with y sides"]
#[num_args(2)]
#[example = "1 6"]
async fn roll(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let amount_of_dice = args.single::<u64>().await?;
    let number_of_eyes = args.single::<u64>().await?;

    if amount_of_dice > 50 {
        return match msg
            .channel_id
            .say(&ctx.http, "Only < 50 dice allowed")
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failure sending message: {:?}", e);
                Err(e.into())
            }
        };
    }

    let mut dice_rolls = Vec::new();
    {
        // prevent rng to be held across an await point
        let mut rng = rand::thread_rng();
        for _ in 0..amount_of_dice {
            dice_rolls.push(rng.gen_range(0, number_of_eyes));
        }
    }

    match msg
        .channel_id
        .say(&ctx.http, &format!("Your roll: {:?}", dice_rolls))
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failure sending message: {:?}", e);
            Err(e.into())
        }
    }
}
