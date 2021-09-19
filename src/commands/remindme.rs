use chrono::Utc;

use tokio::time::sleep;
use twilight_model::{
    application::{
        callback::{CallbackData, InteractionResponse},
        interaction::{application_command::CommandDataOption, ApplicationCommand},
    },
    channel::message::MessageFlags,
};

use crate::{error::TrashyCommandError, models::reminder::Reminder, util, TrashyContext};

pub async fn remindme(
    cmd: Box<ApplicationCommand>,
    ctx: &TrashyContext,
) -> Result<(), TrashyCommandError> {
    let date_option = cmd
        .data
        .options
        .get(0)
        .ok_or(TrashyCommandError::MissingOption)?;

    let duration = match date_option {
        CommandDataOption::String { value, .. } => util::parse_duration_or_date(&value)
            .ok_or(TrashyCommandError::UnknownOption(value.to_string()))?,
        o => return Err(TrashyCommandError::UnknownOption(o.name().to_string())),
    };

    let message = cmd
        .data
        .options
        .get(1)
        .map(|option| match option {
            CommandDataOption::String { value, .. } => Ok(value.to_string()),
            o => Err(TrashyCommandError::UnknownOption(o.name().to_string())),
        })
        .transpose()?;

    let remind_date = Utc::now() + duration;

    let interaction_resp = InteractionResponse::ChannelMessageWithSource(CallbackData {
        allowed_mentions: None,
        components: None,
        content: Some(format!("I will remind you @ {}", remind_date)),
        embeds: Vec::new(),
        flags: Some(MessageFlags::EPHEMERAL),
        tts: None,
    });

    // drop the response here, we want to make a best effort to remind the user in any case, even if something went wrong with the interaction callback
    let _ = ctx
        .http
        .interaction_callback(cmd.id, &cmd.token, &interaction_resp)
        .exec()
        .await;

    let anchor = ctx
        .http
        .create_message(cmd.channel_id)
        .content("Remindme Context")?
        .exec()
        .await?
        .model()
        .await?;

    let user_id = cmd.member.map(|m| m.user.map(|u| u.id)).flatten().ok_or(
        TrashyCommandError::MissingData("missing member/user data for interaction".to_string()),
    )?;

    let reminder = Reminder::create(
        &ctx.db,
        cmd.channel_id,
        user_id,
        anchor.id,
        remind_date,
        &message.unwrap_or_else(String::new),
    )
    .await?;

    sleep(duration.to_std().unwrap()).await;

    Reminder::delete(&ctx.db, reminder.id).await;

    ctx.http
        .create_message(cmd.channel_id)
        .reply(anchor.id)
        .content(&format!(
            "<@{}> i should remind you that: {}",
            user_id, reminder.msg
        ))?
        .exec()
        .await?;

    Ok(())
}

// pub async fn remindme(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
//     let duration = util::parse_duration(&args.single::<String>()?);
//     let pool = get_client(ctx).await?;

//     match duration {
//         None => {
//             std::mem::drop(
//                 msg.reply(ctx, "Unknown time unit. Allowed units are: s,m,h,d,w")
//                     .await,
//             );
//         }
//         Some(duration) => {
//             let defaults = ContentSafeOptions::default();
//             let message = content_safe(&ctx, args.rest().to_string(), &defaults);

//             Reminder::create(
//                 &pool,
//                 *msg.channel_id.as_u64() as i64,
//                 *msg.id.as_u64() as i64,
//                 *msg.author.id.as_u64() as i64,
//                 Utc::now() + duration,
//                 &message,
//             )
//             .await?;

//             std::mem::drop(
//                 msg.react(ctx, ReactionType::Unicode("\u{2705}".to_string()))
//                     .await,
//             );

//             sleep(duration.to_std()?).await;

//             std::mem::drop(Reminder::delete(&pool, *msg.id.as_u64() as i64).await);

//             msg.reply_ping(
//                 ctx,
//                 MessageBuilder::new()
//                     .push("Reminder: ")
//                     .push(message)
//                     .build(),
//             )
//             .await?;
//         }
//     }
//     Ok(())
// }
