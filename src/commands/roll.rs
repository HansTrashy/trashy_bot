use serenity::{
    framework::standard::{
        Args, CommandResult,
        macros::command,
    },
    model::channel::Message,
};
use serenity::prelude::*;
use log::*;
use rand::prelude::*;

#[command]
#[description = "Roll x dice with y sides"]
#[num_args(2)]
#[example = "1 6"]
fn roll(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let amount_of_dice = args.single::<u64>()?;
    let number_of_eyes = args.single::<u64>()?;

    if amount_of_dice > 50 {
        return match msg.channel_id.say(&ctx.http, "Only < 50 dice allowed") {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failure sending message: {:?}", e);
                Err(e.into())
            }
        };
    }

    let mut rng = rand::thread_rng();
    let mut dice_rolls = Vec::new();
    for _ in 0..amount_of_dice {
        dice_rolls.push(rng.gen_range(0, number_of_eyes));
    }

    match msg
        .channel_id
        .say(&ctx.http, &format!("Your roll: {:?}", dice_rolls))
    {
        Ok(_) => Ok(()),
        Err(e) => {
            error!("Failure sending message: {:?}", e);
            Err(e.into())
        }
    }
}
