use crate::models::shiny::Shiny;
use crate::DatabaseConnection;
use serenity::utils::{content_safe, ContentSafeOptions, MessageBuilder};
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
};

//TODO: this need some rework too, shinies are updated by their user_ids across servers, so every server creates probably a new entry and then updates all of them

#[command]
#[description = "Lists Shiny counts"]
#[example("")]
#[only_in("guilds")]
fn list(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write();
    let mut conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply(&ctx, "Could not retrieve the database connection!");
            return Ok(());
        }
    };

    if let Some(server_id) = msg.guild_id {
        let shinys = Shiny::list(&mut *conn, *server_id.as_u64() as i64)?;

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
    let mut conn = data
        .get::<DatabaseConnection>()
        .map(|v| v.get().expect("pool error"))
        .ok_or("Could not retrieve the database connection!")?;

    let respond = |shiny: Shiny| {
        msg.reply(&ctx, format!("Shiny value: {}", shiny.amount))
            .expect("Could not answer");
    };

    // check if user has an entry already
    if let Ok(user_shiny) = Shiny::get(&mut *conn, *msg.author.id.as_u64() as i64) {
        let updated_shiny = Shiny::update(
            &mut *conn,
            *msg.author.id.as_u64() as i64,
            user_shiny.amount + amount,
        )?;

        respond(updated_shiny);
    } else {
        // insert new entry

        if let Some(server_id) = msg.guild_id {
            let new_shiny = Shiny::create(
                &mut *conn,
                *server_id.as_u64() as i64,
                *msg.author.id.as_u64() as i64,
                msg.author.name.to_string(),
                amount,
            )?;
            respond(new_shiny);
        }
    }

    Ok(())
}

#[command]
#[description = "Set the shiny amount of specific user(s)"]
#[example("1000 @HansTrashy")]
#[only_in("guilds")]
#[allowed_roles("Mods")]
#[usage("*amount*")]
fn setshiny(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let amount = args.single::<i64>()?;
    let data = ctx.data.write();
    let mut conn = data
        .get::<DatabaseConnection>()
        .map(|v| v.get().expect("pool error"))
        .ok_or("Could not retrieve the database connection!")?;

    let mut response = Vec::new();

    for user in &msg.mentions {
        // check if user has an entry already
        if let Ok(_user_shiny) = Shiny::get(&mut *conn, *user.id.as_u64() as i64) {
            let updated_shiny = Shiny::update(&mut *conn, *user.id.as_u64() as i64, amount)?;

            response.push(format!("{}: {}", user.name, updated_shiny.amount));
        } else {
            // insert new entry

            if let Some(server_id) = msg.guild_id {
                let new_shiny = Shiny::create(
                    &mut *conn,
                    *server_id.as_u64() as i64,
                    *user.id.as_u64() as i64,
                    user.name.to_string(),
                    amount,
                )?;

                response.push(format!("{}: {}", user.name, new_shiny.amount));
            }
        }
    }

    msg.reply(&ctx, response.join("\n"))
        .expect("Could not answer");

    Ok(())
}

#[command]
#[description = "Removes the shiny amount of specific user(s)"]
#[example("@HansTrashy")]
#[only_in("guilds")]
#[allowed_roles("Mods")]
#[usage("*user1* *user2*")]
fn removeshiny(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.write();
    let mut conn = data
        .get::<DatabaseConnection>()
        .map(|v| v.get().expect("pool error"))
        .ok_or("Could not retrieve the database connection!")?;

    let mut response = Vec::new();

    for user in &msg.mentions {
        // check if user has an entry already
        let updated_shiny = Shiny::delete(&mut *conn, *user.id.as_u64() as i64)?;

        response.push(format!("Removed shinys for {}", user.name));
    }

    msg.reply(&ctx, response.join("\n"))
        .expect("Could not answer");

    Ok(())
}
