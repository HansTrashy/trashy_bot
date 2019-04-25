use crate::RulesState;
use itertools::Itertools;
use serenity::model::channel::ReactionType;
use serenity::utils::{content_safe, ContentSafeOptions};
use std::iter::FromIterator;

command!(de(ctx, msg, _args) {
    let rules = match ctx.data.lock().get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let settings = ContentSafeOptions::default().clean_channel(false);

    rules.lock().de.chars().chunks(1_500).into_iter().for_each(|chunk| {
        msg.author.dm(|m| m.content(content_safe(&String::from_iter(chunk), &settings))).ok();
    });
    let _ = msg.react(ReactionType::Unicode("📬".to_string()));
});

command!(en(ctx, msg, _args) {
    let rules = match ctx.data.lock().get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let settings = ContentSafeOptions::default().clean_channel(false);

    rules.lock().en.chars().chunks(1_500).into_iter().for_each(|chunk| {
        msg.author.dm(|m| m.content(content_safe(&String::from_iter(chunk), &settings))).ok();
    });
    let _ = msg.react(ReactionType::Unicode("📬".to_string()));
});

command!(setde(ctx, msg, args) {
    let rules = match ctx.data.lock().get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    rules.lock().set_de(&args.rest());
    let _ = msg.react(ReactionType::Unicode("👌".to_string()));
});

command!(addde(ctx, msg, args) {
    let rules = match ctx.data.lock().get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let new_rules = format!("{}\n\n{}", rules.lock().de, &args.rest());
    rules.lock().set_de(&new_rules);
    let _ = msg.react(ReactionType::Unicode("👌".to_string()));
});

command!(seten(ctx, msg, args) {
    let rules = match ctx.data.lock().get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    rules.lock().set_en(&args.rest());
    let _ = msg.react(ReactionType::Unicode("👌".to_string()));
});

command!(adden(ctx, msg, args) {
    let rules = match ctx.data.lock().get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let new_rules = format!("{}\n\n{}", rules.lock().en, &args.rest());
    rules.lock().set_de(&new_rules);
    let _ = msg.react(ReactionType::Unicode("👌".to_string()));
});

command!(post(ctx, msg, args) {
    let lang = args.single::<String>().unwrap();
    let rules = match ctx.data.lock().get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let settings = ContentSafeOptions::default().clean_channel(false);
    let lock = rules.lock();

    let rules_text = match lang.as_str() {
        "en" => &lock.en,
        "de" => &lock.de,
        _ => &lock.de,
    };

    rules_text.chars().chunks(1_500).into_iter().for_each(|chunk| {
        msg.channel_id.say(content_safe(&String::from_iter(chunk), &settings)).ok();
    });
});
