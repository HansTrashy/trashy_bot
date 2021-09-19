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
        content: Some(format!("I will remind you @ {}", remind_date)), //TODO: pretty print the date
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
