use crate::models::reaction_role::{self, ReactionRole};
use crate::reaction_roles::State as RRState;
use crate::schema::reaction_roles::dsl::*;
use crate::DatabaseConnection;
use crate::ReactionRolesState;
use crate::Waiter;
use chrono::prelude::*;
use diesel::prelude::*;
use log::{debug, error, info, warn};
use rand::prelude::*;
use serenity::Result as SerenityResult;
use serenity::model::{channel::Message, channel::ReactionType, id::ChannelId, id::MessageId};
use itertools::Itertools;

command!(createrr(ctx, msg, args) {
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    let emoji_arg = args.single::<String>().unwrap();
    let role_group_arg = args.single::<String>().unwrap();
    let role_arg = args.rest();

    if let Some(guild) =  msg.guild() {
        if let Some(role) = guild.read().role_by_name(&role_arg) {
            reaction_role::create_reaction_role(
                &*conn.lock(),
                *msg
                    .channel()
                    .unwrap()
                    .guild()
                    .unwrap()
                    .read()
                    .guild_id
                    .as_u64() as i64,
                *role.id.as_u64() as i64,
                role_arg.to_string(),
                role_group_arg,
                emoji_arg,
            );
            let _ = msg.reply("Added rr!");
        }
    }
});

command!(removerr(ctx, msg, args) {
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };

    let emoji_arg = args.single::<String>().unwrap();
    let role_arg = args.rest();
    dbg!(&role_arg);

    if let Some(guild) = msg.guild() {
        debug!("Some guild found");
        if let Some(role) = guild.read().role_by_name(&role_arg) {
            debug!("Role found: {:?}", &role);
            diesel::delete(reaction_roles.filter(emoji.eq(emoji_arg)).filter(role_id.eq(*role.id.as_u64() as i64))).execute(&*conn.lock()).expect("could not delete reaction role");
            let _ = msg.reply("deleted rr!");
        }
    }
});

command!(listrr(ctx, msg, _args) {
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };
    use crate::schema::reaction_roles::dsl::*;
    use diesel::prelude::*;

    let results = reaction_roles.load::<ReactionRole>(&*conn.lock()).expect("could not retrieve reaction roles");

    let mut output = String::new();
    for r in results {
        output.push_str(&format!("{} | {} | {}\n", r.emoji, r.role_group, r.role_name));
    }

    let _ = msg.channel_id.send_message(|m| m.embed(|e| 
                e.description(&output)
                .color((0,120,220))));
});

command!(postrrgroups(ctx, msg, _args) {
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.clone(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };

    use crate::schema::reaction_roles::dsl::*;
    use diesel::prelude::*;

    let mut results = reaction_roles.load::<ReactionRole>(&*conn.lock()).expect("Could not retrieve rr");
    results.sort_by_key(|r| r.role_group.to_owned());
    // post a message for each group and react under them with the respective emojis

    let rr_message_ids: Vec<u64> = results.into_iter().group_by(|r| r.role_group.to_owned()).into_iter().map(|(key, group)| {
        let collected_group = group.collect::<Vec<ReactionRole>>();
        let mut rendered_roles = String::new();
        for r in &collected_group {
            rendered_roles.push_str(&format!("{} | {}\n", r.emoji, r.role_name));
        }

        let group_message = msg.channel_id.send_message(|m| m.embed(|e|
            e.title(&format!("Rollengruppe: {}", key)).description(rendered_roles)
            .color((0,120,220))
        ));

        if let Ok(m) = group_message {
            for r in collected_group {
                let _ = m.react(ReactionType::Unicode(r.emoji));
            }
            Some(*m.id.as_u64())
        } else {
            None
        }
    }).filter_map(|x| x).collect();


    // set the corresponding rr state

    match data.get::<ReactionRolesState>() {
        Some(v) => {
            *v.lock() = RRState::set(*msg.channel_id.as_u64(), rr_message_ids);
        },
        None => panic!("No reaction role state available!"),
    }
});