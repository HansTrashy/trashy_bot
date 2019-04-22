use crate::RulesState;
use serenity::model::channel::ReactionType;
use serenity::utils::{content_safe, ContentSafeOptions};

command!(de(ctx, msg, _args) {
    let rules = match ctx.data.lock().get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let settings = ContentSafeOptions::default().clean_channel(false);

    let _ = msg.author.dm(|m| m.content(content_safe(&*rules.lock().de, &settings)));
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

    let _ = msg.author.dm(|m| m.content(content_safe(&*rules.lock().en, &settings)));
    let _ = msg.react(ReactionType::Unicode("📬".to_string()));
});

command!(setde(ctx, msg, _args) {
    let rules = match ctx.data.lock().get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    rules.lock().set_de(&msg.content);
    let _ = msg.react(ReactionType::Unicode("👌".to_string()));
});

command!(seten(ctx, msg, _args) {
    let rules = match ctx.data.lock().get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    rules.lock().set_en(&msg.content);
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
    let lock = rules.lock();

    let _ = msg.channel_id.say(match lang.as_str() {
        "en" => &lock.en,
        "de" => &lock.de,
        _ => &*lock.de,
    });
});
