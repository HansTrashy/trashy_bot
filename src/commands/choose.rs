use rand::prelude::*;
// use rand::seq::SliceRandom;

command!(choose(_ctx, msg, args) {
    let mut rng = rand::thread_rng();

    if args.len() < 2 {
        if let Err(why) = msg.channel_id.say("You have to give at least 2 options") {
            println!("Error sending message: {:?}", why);
        }
        return Ok(());
    }

    let chosen = args.iter::<String>().choose(&mut rng).unwrap().unwrap();

    if let Err(why) = msg.channel_id.say(chosen) {
        println!("Error sending message: {:?}", why);
    }
});
