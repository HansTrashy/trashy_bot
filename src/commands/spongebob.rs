use twilight_model::{
    application::{
        callback::{CallbackData, InteractionResponse},
        interaction::{application_command::CommandDataOption, ApplicationCommand},
    },
    channel::embed::{Embed, EmbedAuthor},
};

use crate::{error::TrashyCommandError, TrashyContext};

pub async fn sponge(
    cmd: Box<ApplicationCommand>,
    ctx: &TrashyContext,
) -> Result<(), TrashyCommandError> {
    let spongify_this = match &cmd.data.options.get(0) {
        Some(CommandDataOption::String { value, .. }) => value,
        _ => {
            tracing::error!("wrong or no command option dataype received!");
            return Ok(());
        }
    };

    let spongified = spongify_this
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if (i % 2) == 0 {
                c.to_uppercase().to_string()
            } else {
                c.to_lowercase().to_string()
            }
        })
        .collect::<String>();

    let embed = Embed {
        author: Some(EmbedAuthor {
            icon_url: Some(
                "https://cdn.discordapp.com/emojis/598837367343808532.png?v=1".to_string(),
            ),
            name: Some("Spongebob".to_string()),
            proxy_icon_url: None,
            url: None,
        }),
        color: Some(0xFFFF00),
        description: Some(spongified),
        fields: Vec::new(),
        footer: None,
        image: None,
        kind: "rich".to_string(),
        provider: None,
        thumbnail: None,
        timestamp: None,
        title: None,
        url: None,
        video: None,
    };

    let interaction_resp = InteractionResponse::ChannelMessageWithSource(CallbackData {
        allowed_mentions: None,
        components: None,
        content: None,
        embeds: vec![embed],
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
