use crate::models::reaction_role::ReactionRole;
use crate::reaction_roles::State as RRState;
use crate::DatabaseConnection;
use crate::ReactionRolesState;
use itertools::Itertools;
use log::*;
use serenity::model::channel::ReactionType;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

#[command]
#[allowed_roles("Mods")]
#[description = "Creates a new reaction role"]
#[example = "🧀 group_name role_name"]
pub async fn create(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let mut conn = data
        .get::<DatabaseConnection>()
        .map(|v| v.get().expect("pool error"))
        .ok_or("Could not retrieve the database connection!")?;
    let emoji_arg = args.single::<String>().await?;
    let role_group_arg = args.single::<String>().await?;
    let role_arg = args.rest();

    if let Some(guild) = msg.guild(&ctx.cache).await {
        if let Some(role) = guild.read().await.role_by_name(&role_arg) {
            ReactionRole::create(
                &mut *conn,
                *msg.channel(&ctx.cache)
                    .await
                    .ok_or("no channel")?
                    .guild()
                    .ok_or("no guild")?
                    .read()
                    .await
                    .guild_id
                    .as_u64() as i64,
                *role.id.as_u64() as i64,
                role_arg.to_string(),
                role_group_arg,
                emoji_arg,
            );
            let _ = msg.reply(&ctx, "Added rr!");
        }
    }
    Ok(())
}

#[command]
#[allowed_roles("Mods")]
#[description = "Removes a reaction role"]
#[example = "🧀 role_name"]
pub async fn remove(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let mut conn = data
        .get::<DatabaseConnection>()
        .map(|v| v.get().expect("pool error"))
        .ok_or("Could not retrieve the database connection!")?;

    let emoji_arg = args.single::<String>().await.unwrap();
    let role_arg = args.rest();
    dbg!(&role_arg);

    if let Some(guild) = msg.guild(&ctx.cache).await {
        debug!("Some guild found");
        if let Some(role) = guild.read().await.role_by_name(&role_arg) {
            debug!("Role found: {:?}", &role);
            let _ = ReactionRole::delete(
                &mut *conn,
                *msg.guild_id.unwrap().as_u64() as i64,
                *role.id.as_u64() as i64,
            );
            let _ = msg.reply(&ctx, "deleted rr!");
        }
    }
    Ok(())
}

#[command]
#[allowed_roles("Mods")]
#[description = "Lists all reaction roles"]
pub async fn list(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let mut conn = data
        .get::<DatabaseConnection>()
        .map(|v| v.get().expect("pool error"))
        .ok_or("Could not retrieve the database connection!")?;

    let results = ReactionRole::list(&mut *conn)?;

    let mut output = String::new();
    for r in results {
        output.push_str(&format!(
            "{} | {} | {}\n",
            r.emoji, r.role_group, r.role_name
        ));
    }

    let _ = msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| e.description(&output).color((0, 120, 220)))
    });
    Ok(())
}

#[command]
#[allowed_roles("Mods")]
#[description = "Posts the reaction role groups"]
pub async fn postgroups(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let mut conn = data
        .get::<DatabaseConnection>()
        .map(|v| v.get().expect("pool error"))
        .ok_or("Could not retrieve the database connection!")?;

    let mut results = ReactionRole::list(&mut *conn)?;
    results.sort_by_key(|r| r.role_group.to_owned());
    // post a message for each group and react under them with the respective emojis

    let rr_message_ids: Vec<u64> = results
        .into_iter()
        .group_by(|r| r.role_group.to_owned())
        .into_iter()
        .map(|(key, group)| {
            let collected_group = group.collect::<Vec<ReactionRole>>();
            let mut rendered_roles = String::new();
            for r in &collected_group {
                rendered_roles.push_str(&format!("{} | {}\n", r.emoji, r.role_name));
            }

            let group_message = msg
                .channel_id
                .send_message(&ctx, |m| {
                    m.embed(|e| {
                        e.title(&format!("Rollengruppe: {}", key))
                            .description(rendered_roles)
                            .color((0, 120, 220))
                    })
                })
                .await;

            if let Ok(m) = group_message {
                for r in collected_group {
                    let _ = m.react(&ctx, ReactionType::Unicode(r.emoji));
                }
                Some(*m.id.as_u64())
            } else {
                None
            }
        })
        .filter_map(|x| x)
        .collect();

    // set the corresponding rr state

    match data.get::<ReactionRolesState>() {
        Some(v) => {
            *v.lock().await = RRState::set(*msg.channel_id.as_u64(), rr_message_ids);
        }
        None => panic!("No reaction role state available!"),
    }
    Ok(())
}
