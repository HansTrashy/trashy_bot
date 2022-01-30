use tokio::time::sleep;
use twilight_model::{
    application::{
        callback::{CallbackData, InteractionResponse},
        interaction::{application_command::CommandOptionValue, ApplicationCommand},
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

    let duration = if let CommandOptionValue::String(value) = &date_option.value {
        util::parse_duration_or_date(&value)
            .ok_or(TrashyCommandError::UnknownOption(value.to_string()))?
    } else {
        return Err(TrashyCommandError::UnknownOption(
            date_option.name.to_string(),
        ));
    };

    let message = cmd
        .data
        .options
        .get(1)
        .map(|option| {
            if let CommandOptionValue::String(v) = &option.value {
                Ok(v.to_string())
            } else {
                Err(TrashyCommandError::UnknownOption(option.name.to_string()))
            }
        })
        .transpose()?;

    let remind_date = time::OffsetDateTime::now_utc() + duration;

    let interaction_resp = InteractionResponse::ChannelMessageWithSource(CallbackData {
        allowed_mentions: None,
        components: None,
        content: Some(format!("I will remind you @ {}", remind_date)), //TODO: pretty print the date
        embeds: None,
        flags: Some(MessageFlags::EPHEMERAL),
        tts: None,
    });

    // drop the response here, we want to make a best effort to remind the user in any case, even if something went wrong with the interaction callback
    let _ = ctx
        .http
        .interaction(ctx.app_id)
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

    sleep(duration.try_into().unwrap()).await;

    match Reminder::delete(&ctx.db, reminder.id).await {
        Ok(_) => (),
        Err(e) => tracing::error!(?e, "deleting reminder failed"),
    };

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
