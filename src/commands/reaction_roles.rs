use crate::models::reaction_role::ReactionRole;
use crate::reaction_roles::State as RRState;
use crate::util::get_client;
use crate::DatabasePool;
use crate::ReactionRolesState;
use serenity::model::channel::ReactionType;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use std::collections::HashMap;
use tracing::debug;

#[command]
#[allowed_roles("Mods")]
#[description = "Testing function"]
pub async fn testing(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let role_arg = args.single::<String>()?;

    let guild = msg.guild(&ctx).await.ok_or("No Guild found")?;
    debug!(role_arg = ?role_arg, "trying to find role");
    let role = guild.role_by_name(&role_arg).ok_or("Role not found")?;
    debug!(role = ?role, "found role");

    Ok(())
}

#[command]
#[allowed_roles("Mods")]
#[description = "Creates a new reaction role"]
#[example = "🧀 group_name role_name"]
pub async fn create(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let emoji_arg = args.single::<String>()?;
    let role_group_arg = args.single::<String>()?;
    let role_arg = args.single::<String>()?;

    let guild = msg.guild(&ctx).await.ok_or("No Guild found")?;
    debug!("trying to find role: '{:?}'", &role_arg);
    let role = guild.role_by_name(&role_arg).ok_or("Role not found")?;

    ReactionRole::create(
        &mut *get_client(&ctx).await?,
        *msg.channel(&ctx.cache)
            .await
            .ok_or("no channel")?
            .guild()
            .ok_or("no guild")?
            .guild_id
            .as_u64() as i64,
        *role.id.as_u64() as i64,
        role_arg.to_string(),
        role_group_arg,
        emoji_arg,
    )
    .await?;

    let _ = msg.reply(ctx, "Added rr!").await;

    Ok(())
}

#[command]
#[allowed_roles("Mods")]
#[description = "Removes a reaction role"]
#[example = "🧀 role_name"]
pub async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let emoji_arg = args.single::<String>()?;
    let role_arg = args.rest();
    dbg!(&role_arg);

    if let Some(guild) = msg.guild(&ctx.cache).await {
        debug!("Some guild found");
        if let Some(role) = guild.role_by_name(role_arg) {
            debug!("Role found: {:?}", &role);
            ReactionRole::delete(
                &mut *get_client(&ctx).await?,
                *msg.guild_id.unwrap().as_u64() as i64,
                *role.id.as_u64() as i64,
            )
            .await?;
            msg.reply(ctx, "deleted rr!").await?;
        }
    }
    Ok(())
}

#[command]
#[allowed_roles("Mods")]
#[description = "Lists all reaction roles"]
pub async fn list(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let results = ReactionRole::list(&mut *get_client(&ctx).await?).await?;

    let mut output = String::new();
    for r in results {
        output.push_str(&format!(
            "{} | {} | {}\n",
            r.emoji, r.role_group, r.role_name
        ));
    }

    msg.channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| e.description(&output).color((0, 120, 220)))
        })
        .await?;
    Ok(())
}

#[command]
#[allowed_roles("Mods")]
#[description = "Posts the reaction role groups"]
pub async fn postgroups(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut results = ReactionRole::list(&mut *get_client(&ctx).await?).await?;
    results.sort_by_key(|r| r.role_group.to_owned());
    // post a message for each group and react under them with the respective emojis

    let reaction_groups = results.into_iter().fold(HashMap::new(), |mut acc, role| {
        let entry = acc.entry(role.role_group.clone()).or_insert_with(Vec::new);
        entry.push(role);

        acc
    });

    let mut rr_message_ids = Vec::new();
    for (reaction_group_name, roles) in reaction_groups {
        let mut rendered_roles = String::new();
        for r in &roles {
            rendered_roles.push_str(&format!("{} | {}\n", r.emoji, r.role_name));
        }

        let group_message = msg
            .channel_id
            .send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title(&format!("Rollengruppe: {}", reaction_group_name))
                        .description(rendered_roles)
                        .color((0, 120, 220))
                })
            })
            .await?;

        rr_message_ids.push(*group_message.id.as_u64());

        for r in &roles {
            group_message
                .react(ctx, ReactionType::Unicode(r.emoji.clone()))
                .await?;
        }
    }

    // set the corresponding rr state

    match ctx.data.read().await.get::<ReactionRolesState>() {
        Some(v) => {
            *v.lock().await = RRState::set(*msg.channel_id.as_u64(), rr_message_ids);
        }
        None => panic!("No reaction role state available!"),
    }
    Ok(())
}
