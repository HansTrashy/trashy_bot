use crate::models::reaction_role::{self, ReactionRole};
use crate::reaction_roles::State as RRState;
use crate::schema::reaction_roles::dsl::*;
use crate::DatabaseConnection;
use crate::ReactionRolesState;
use diesel::prelude::*;
use log::*;
use serenity::model::channel::ReactionType;
use itertools::Itertools;
use serenity::{
    framework::standard::{
        Args, CommandResult,
        macros::command,
    },
    model::channel::Message,
};
use serenity::prelude::*;
use log::*;

#[command]
#[allowed_roles("Mods")]
#[description = "Creates a new reaction role"]
#[example = "🧀 group_name role_name"]
pub fn create(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let emoji_arg = args.single::<String>()?;
    let role_group_arg = args.single::<String>()?;
    let role_arg = args.rest();

    if let Some(guild) = msg.guild(&ctx.cache) {
        if let Some(role) = guild.read().role_by_name(&role_arg) {
            reaction_role::create_reaction_role(
                &conn,
                *msg.channel(&ctx.cache)
                    .ok_or("no channel")?
                    .guild()
                    .ok_or("no guild")?
                    .read()
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
pub fn remove(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };

    let emoji_arg = args.single::<String>().unwrap();
    let role_arg = args.rest();
    dbg!(&role_arg);

    if let Some(guild) = msg.guild(&ctx.cache) {
        debug!("Some guild found");
        if let Some(role) = guild.read().role_by_name(&role_arg) {
            debug!("Role found: {:?}", &role);
            diesel::delete(
                reaction_roles
                    .filter(emoji.eq(emoji_arg))
                    .filter(role_id.eq(*role.id.as_u64() as i64)),
            )
            .execute(&conn)
            .expect("could not delete reaction role");
            let _ = msg.reply(&ctx, "deleted rr!");
        }
    }
    Ok(())
}

#[command]
#[allowed_roles("Mods")]
#[description = "Lists all reaction roles"]
pub fn list(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };
    use crate::schema::reaction_roles::dsl::*;
    use diesel::prelude::*;

    let results = reaction_roles
        .load::<ReactionRole>(&conn)
        .expect("could not retrieve reaction roles");

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
pub fn postgroups(ctx: &mut Context, msg: &Message, _args: Args) -> CommandResult {
    let data = ctx.data.read();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };

    use crate::schema::reaction_roles::dsl::*;
    use diesel::prelude::*;

    let mut results = reaction_roles
        .load::<ReactionRole>(&conn)
        .expect("Could not retrieve rr");
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

            let group_message = msg.channel_id.send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title(&format!("Rollengruppe: {}", key))
                        .description(rendered_roles)
                        .color((0, 120, 220))
                })
            });

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
            *v.lock() = RRState::set(*msg.channel_id.as_u64(), rr_message_ids);
        }
        None => panic!("No reaction role state available!"),
    }
    Ok(())
}
