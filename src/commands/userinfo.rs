use chrono::prelude::*;
use chrono::Utc;
use futures::future::join_all;
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};

pub struct UserInfo {
    pub created_at: String,
    pub created_at_ago: i64,
    pub member: Option<MemberInfo>,
}

pub struct MemberInfo {
    pub nick: String,
    pub joined_at: String,
    pub joined_at_ago: String,
    pub roles: Vec<String>,
}

#[command]
#[description = "Display information about the user"]
#[usage = "*user_mention*"]
#[only_in("guilds")]
#[example = "@HansTrashy"]
pub async fn userinfo(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let user = msg.mentions.get(0).ok_or("No user mentioned")?;

    let mut user_info = UserInfo {
        created_at: user.created_at().format("%d.%m.%Y %H:%M:%S").to_string(),
        created_at_ago: Utc::now()
            .signed_duration_since(user.created_at())
            .num_days(),
        member: None,
    };

    if let Some(guild_id) = msg.guild_id {
        let member = guild_id.member(ctx, user.id).await?;
        let special_case =
            if user.id == 200_009_451_292_459_011 && guild_id == 217_015_995_385_118_721 {
                Some(Utc.ymd(2017, 5, 26).and_hms(8, 56, 0))
            } else {
                None
            };
        let joined_at = if let Some(special) = special_case {
            special.format("%d.%m.%Y %H:%M:%S").to_string()
        } else {
            member
                .joined_at
                .map(|time| time.format("%d.%m.%Y %H:%M:%S").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        };
        let joined_at_ago = if let Some(special) = special_case {
            Utc::now()
                .signed_duration_since(special)
                .num_days()
                .to_string()
        } else {
            member
                .joined_at
                .map(|time| {
                    Utc::now()
                        .signed_duration_since(time)
                        .num_days()
                        .to_string()
                })
                .unwrap_or_else(|| "Unknown".to_string())
        };

        let roles = join_all(member.roles.iter().map(|r| r.to_role_cached(&ctx.cache)))
            .await
            .into_iter()
            .filter_map(|r| r.map(|r| r.name))
            .collect();

        let member_info = MemberInfo {
            nick: member
                .nick
                .unwrap_or_else(|| format!("{}#{}", user.name, user.discriminator)),
            joined_at,
            joined_at_ago,
            roles,
        };
        user_info.member = Some(member_info);
    }

    let default = "Unknown".to_string();

    let information_body = format!(
        "**Joined discord:** {} ({} days ago)\n\n**Joined this server:** {} ({} days ago)\n\n**Roles:** {}",
        user_info.created_at,
        user_info.created_at_ago,
        user_info
            .member
            .as_ref()
            .map_or(&default, |m| &m.joined_at),
        user_info
            .member
            .as_ref()
            .map_or(&default, |m| &m.joined_at_ago),
        user_info
            .member
            .as_ref()
            .map_or(default.clone(), |m| m.roles.join(", ")),
    );

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.author(|a| {
                    a.name(&user.name)
                        .icon_url(&user.static_avatar_url().unwrap_or_default())
                })
                .color((0, 120, 220))
                .description(&information_body)
                .footer(|f| {
                    f.text(&format!(
                        "{}#{} | id: {}",
                        user.name, user.discriminator, &user.id,
                    ))
                })
            })
        })
        .await?;
    Ok(())
}
