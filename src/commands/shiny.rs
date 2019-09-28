use serde::Deserialize;
use serenity::{
    prelude::*,
    framework::standard::{Args, CommandResult, macros::command},
    model::prelude::*,
};
use serenity::utils::{content_safe, ContentSafeOptions, MessageBuilder};
use crate::models::shiny::{Shiny, NewShiny};
use crate::schema::shinys::dsl;
use crate::DatabaseConnection;
use diesel::prelude::*;

#[command]
#[description = "Lists Shiny counts"]
#[example("")]
#[only_in("guilds")]
fn list(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };

    if let Some(server_id) = msg.guild_id {
        let shinys = dsl::shinys
            .filter(dsl::server_id.eq(*server_id.as_u64() as i64))
            .load::<Shiny>(&conn)
            .expect("Could not load shinies");

        let mut content = MessageBuilder::new();
        content.push_line("Shinys Tracked");

        for s in shinys {
            content.push_line(format!("{}: {}", s.username, s.amount));
        }

        let _ = msg.channel_id.send_message(&ctx, |m| {
            m.embed(|e| e.description(content.build()).color((0, 120, 220)))
        });
    }

    Ok(())
}

#[command]
#[description = "Increases your shiny charity count"]
#[example("1000")]
#[only_in("guilds")]
#[usage("*amount*")]
fn shiny(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let amount = args.single::<i64>()?;
    let mut data = ctx.data.write();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };

    let respond = |shiny: Shiny| {
        msg.reply(&ctx, format!("Shiny value: {}", shiny.amount))
            .expect("Could not answer");
    };

    // check if user has an entry already
    if let Ok(user_shiny) = dsl::shinys
        .filter(dsl::user_id.eq(*msg.author.id.as_u64() as i64))
        .first::<Shiny>(&conn)
    {
        let updated_shiny =
            diesel::update(dsl::shinys.filter(dsl::user_id.eq(*msg.author.id.as_u64() as i64)))
                .set(dsl::amount.eq(user_shiny.amount + amount))
                .get_result::<Shiny>(&conn)
                .expect("could not update shiny");

        respond(updated_shiny);
    } else {
        // insert new entry

        if let Some(server_id) = msg.guild_id {
            let new_shiny = diesel::insert_into(dsl::shinys)
                .values(&NewShiny {
                    user_id: *msg.author.id.as_u64() as i64,
                    username: msg.author.name.to_string(),
                    server_id: *server_id.as_u64() as i64,
                    amount,
                })
                .get_result::<Shiny>(&conn)
                .expect("Could not insert new amount");
            respond(new_shiny);
        }
    }

    Ok(())
}
