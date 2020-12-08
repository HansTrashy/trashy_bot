use crate::models::shiny::Shiny;
use crate::util::get_client;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::*,
    prelude::*,
    utils::MessageBuilder,
};

//TODO: this need some rework too, shinies are updated by their user_ids across servers, so every server creates probably a new entry and then updates all of them

#[command]
#[description = "List shiny counts"]
#[only_in("guilds")]
async fn list(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    if let Some(server_id) = msg.guild_id {
        let pool = get_client(&ctx).await?;
        let shinys = Shiny::list(&pool, *server_id.as_u64() as i64).await?;

        let mut content = MessageBuilder::new();
        content.push_line("Shinys Tracked");

        for s in shinys {
            content.push_line(format!("{}: {}", s.username, s.amount));
        }

        let _ = msg
            .channel_id
            .send_message(&ctx, |m| {
                m.embed(|e| e.description(content.build()).color((0, 120, 220)))
            })
            .await;
    }

    Ok(())
}

#[command]
#[description = "Increase your shiny charity count"]
#[example("1000")]
#[only_in("guilds")]
#[usage("*amount*")]
async fn shiny(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let amount = args.single::<i64>()?;
    let pool = get_client(&ctx).await?;

    if let Ok(user_shiny) = Shiny::get(&pool, *msg.author.id.as_u64() as i64).await {
        let updated_shiny = Shiny::update(
            &pool,
            *msg.author.id.as_u64() as i64,
            user_shiny.amount + amount,
        )
        .await?;

        respond(&ctx, msg, updated_shiny).await;
    } else if let Some(server_id) = msg.guild_id {
        let new_shiny = Shiny::create(
            &pool,
            *server_id.as_u64() as i64,
            *msg.author.id.as_u64() as i64,
            msg.author.name.to_string(),
            amount,
        )
        .await?;
        respond(&ctx, msg, new_shiny).await;
    }

    Ok(())
}

async fn respond(ctx: &Context, msg: &Message, shiny: Shiny) {
    msg.reply(ctx, format!("Shiny value: {}", shiny.amount))
        .await
        .expect("Could not answer");
}

#[command]
#[description = "Set the shiny amount of specific user(s)"]
#[example("1000 @HansTrashy")]
#[only_in("guilds")]
#[allowed_roles("Mods")]
#[usage("*amount* *user_mention*")]
async fn setshiny(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let amount = args.single::<i64>()?;
    let pool = get_client(&ctx).await?;

    let mut response = Vec::new();

    for user in &msg.mentions {
        // check if user has an entry already
        if let Ok(_user_shiny) = Shiny::get(&pool, *user.id.as_u64() as i64).await {
            let updated_shiny = Shiny::update(&pool, *user.id.as_u64() as i64, amount).await?;

            response.push(format!("{}: {}", user.name, updated_shiny.amount));
        } else {
            // insert new entry

            if let Some(server_id) = msg.guild_id {
                let new_shiny = Shiny::create(
                    &pool,
                    *server_id.as_u64() as i64,
                    *user.id.as_u64() as i64,
                    user.name.to_string(),
                    amount,
                )
                .await?;

                response.push(format!("{}: {}", user.name, new_shiny.amount));
            }
        }
    }

    msg.reply(ctx, response.join("\n"))
        .await
        .expect("Could not answer");

    Ok(())
}

#[command]
#[description = "Removes the shiny amount of specific user(s)"]
#[example("@HansTrashy")]
#[only_in("guilds")]
#[allowed_roles("Mods")]
#[usage("*user_mention_1* *user_mention_2*")]
async fn removeshiny(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let mut response = Vec::new();
    let pool = get_client(&ctx).await?;

    for user in &msg.mentions {
        // check if user has an entry already
        let _updated_shiny = Shiny::delete(&pool, *user.id.as_u64() as i64).await?;

        response.push(format!("Removed shinys for {}", user.name));
    }

    msg.reply(ctx, response.join("\n"))
        .await
        .expect("Could not answer");

    Ok(())
}
