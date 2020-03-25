use crate::RulesState;
use itertools::Itertools;
use log::*;
use serenity::model::channel::ReactionType;
use serenity::prelude::*;
use serenity::utils::{content_safe, ContentSafeOptions};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use std::iter::FromIterator;

#[command]
#[description = "Sends you the Rules in German"]
#[num_args(0)]
#[only_in("guilds")]
pub async fn de(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let rules = match ctx.data.read().await.get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let settings = ContentSafeOptions::default().clean_channel(false);

    rules
        .lock()
        .await
        .de
        .chars()
        .chunks(1_500)
        .into_iter()
        .for_each(|chunk| {
            msg.author
                .dm(&ctx, |m| {
                    m.content(content_safe(&ctx.cache, &String::from_iter(chunk), &settings).await)
                })
                .await
                .ok();
        });
    let _ = msg.react(&ctx, ReactionType::Unicode("📬".to_string()));
    Ok(())
}

#[command]
#[description = "Sends you the rules in english"]
#[num_args(0)]
#[only_in("guilds")]
pub async fn en(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let rules = match ctx.data.read().await.get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let settings = ContentSafeOptions::default().clean_channel(false);

    rules
        .lock()
        .await
        .en
        .chars()
        .chunks(1_500)
        .into_iter()
        .for_each(|chunk| {
            msg.author
                .dm(&ctx, |m| {
                    m.content(content_safe(&ctx.cache, &String::from_iter(chunk), &settings).await)
                })
                .await
                .ok();
        });
    msg.react(&ctx, ReactionType::Unicode("📬".to_string())).await?;
    Ok(())
}

#[command]
#[allowed_roles("Mods")]
#[description = "Sets the rules"]
#[only_in("guilds")]
pub async fn setde(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let rules = match ctx.data.read().await.get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    rules.lock().await.set_de(args.rest());
    msg.react(&ctx, ReactionType::Unicode("✅".to_string())).await?;
    Ok(())
}

#[command]
#[allowed_roles("Mods")]
#[description = "Adds to the rules"]
#[only_in("guilds")]
pub async fn addde(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let rules = match ctx.data.read().await.get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let new_rules = format!("{}\n\n{}", rules.lock().await.de, &args.rest());
    rules.lock().await.set_de(&new_rules);
    let _ = msg.react(&ctx, ReactionType::Unicode("✅".to_string()));
    Ok(())
}

#[command]
#[allowed_roles("Mods")]
#[description = "Sets the rules"]
#[only_in("guilds")]
pub async fn seten(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let rules = match ctx.data.read().await.get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    rules.lock().await.set_en(&args.rest());
    let _ = msg.react(&ctx, ReactionType::Unicode("✅".to_string()));
    Ok(())
}

#[command]
#[allowed_roles("Mods")]
#[description = "Adds to the rules"]
#[only_in("guilds")]
pub async fn adden(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let rules = match ctx.data.read().await.get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let new_rules = format!("{}\n\n{}", rules.lock().await.en, &args.rest());
    rules.lock().await.set_en(&new_rules);
    let _ = msg.react(&ctx, ReactionType::Unicode("✅".to_string()));
    Ok(())
}

#[command]
#[allowed_roles("Mods")]
#[description = "The bot will post the rules into the channel"]
#[num_args(1)]
#[example = "de"]
#[only_in("guilds")]
pub async fn post(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let lang = args.single::<String>().await?;
    let rules = match ctx.data.read().await.get::<RulesState>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let settings = ContentSafeOptions::default().clean_channel(false);
    let lock = rules.lock().await;

    let rules_text = match lang.as_str() {
        "en" => &lock.en,
        _ => &lock.de,
    };

    rules_text
        .chars()
        .chunks(1_500)
        .into_iter()
        .for_each(|chunk| {
            msg.channel_id
                .say(
                    &ctx,
                    content_safe(&ctx.cache, &String::from_iter(chunk), &settings).await,
                )
                .await
                .ok();
        });
    Ok(())
}
