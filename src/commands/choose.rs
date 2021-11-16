use std::ops::DerefMut;

use rand::prelude::*;
use twilight_model::application::{
    callback::{CallbackData, InteractionResponse},
    interaction::{application_command::CommandOptionValue, ApplicationCommand},
};

use crate::{error::TrashyCommandError, TrashyContext};

pub async fn choose(
    cmd: Box<ApplicationCommand>,
    ctx: &TrashyContext,
) -> Result<(), TrashyCommandError> {
    let options = cmd
        .data
        .options
        .get(0)
        .map(|option| {
            if let CommandOptionValue::String(v) = &option.value {
                v.split_whitespace().collect()
            } else {
                Vec::new()
            }
        })
        .unwrap_or_else(Vec::new);

    let how_many = cmd
        .data
        .options
        .get(1)
        .map(|option| {
            if let CommandOptionValue::Integer(v) = &option.value {
                *v
            } else {
                1
            }
        })
        .unwrap_or(1);

    let chosen = {
        let rng = &mut ctx.rng.lock().await;
        let mut_rng = rng.deref_mut();

        (0..how_many)
            .filter_map(|_| options.choose(mut_rng))
            .copied()
            .collect::<Vec<_>>()
    };

    let interaction_resp = InteractionResponse::ChannelMessageWithSource(CallbackData {
        allowed_mentions: None,
        components: None,
        content: Some(chosen.join(", ")),
        embeds: Vec::new(),
        flags: None,
        tts: None,
    });

    let resp = ctx
        .http
        .interaction_callback(cmd.id, &cmd.token, &interaction_resp)
        .exec()
        .await;
    tracing::debug!(?resp);

    Ok(())
}
