use rand::prelude::*;

command!(roll(_ctx, msg, args) {
    let amount_of_dice = args.single::<u32>().unwrap();
    let number_of_eyes = args.single::<u32>().unwrap();

    if amount_of_dice > 50 {
        if let Err(why) = msg.channel_id.say("Only < 50 dice allowed") {
            println!("Error sending message: {:?}", why);
        }
        return Ok(());
    }

    let mut rng = rand::thread_rng();
    let mut dice_rolls = Vec::new();
    for _ in 0..amount_of_dice {
        dice_rolls.push(rng.gen_range(0, number_of_eyes));
    }

    if let Err(why) = msg.channel_id.say(&format!("Your roll: {:?}", dice_rolls)) {
        println!("Error sending message: {:?}", why);
    }
});
