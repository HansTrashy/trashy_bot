use crate::models::reaction_role::{self, ReactionRole};
use crate::schema::reaction_roles::dsl::*;
use crate::DatabaseConnection;
use crate::Waiter;
use chrono::prelude::*;
use diesel::prelude::*;
use rand::prelude::*;
use serenity::model::{channel::Message, channel::ReactionType, id::ChannelId, id::MessageId};

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
    let role_id_arg = args.single::<u64>().unwrap();

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
            role_id_arg as i64,
            emoji_arg,
        );
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
    let role_id_arg = args.single::<u64>().unwrap();

    diesel::delete(reaction_roles.filter(emoji.eq(emoji_arg)).filter(role_id.eq(role_id_arg as i64))).execute(&*conn.lock()).expect("could not delete reaction role");
});

command!(listrr(ctx, msg, args) {
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
        output.push_str(&format!("{} | {}", r.emoji, r.role_id));
    }

    let _ = msg.channel_id.send_message(|m| m.embed(|e| 
                e.description(&output)
                .color((0,120,220))));
});
