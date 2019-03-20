use rand::prelude::*;
// use rand::seq::SliceRandom;
use crate::util::sanitize_for_other_bot_commands;
use serenity::utils::{content_safe, ContentSafeOptions};

command!(choose(_ctx, msg, args) {
    let mut rng = rand::thread_rng();

    let settings = ContentSafeOptions::default().clean_channel(false);

    if args.len() < 2 {
        if let Err(why) = msg.channel_id.say("You have to give at least 2 options") {
            println!("Error sending message: {:?}", why);
        }
        return Ok(());
    }

    let chosen = args.iter::<String>().choose(&mut rng).unwrap().unwrap();

    if let Err(why) = msg.channel_id.say(content_safe(&sanitize_for_other_bot_commands(&chosen), &settings)) {
        println!("Error sending message: {:?}", why);
    }
});
