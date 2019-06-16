use serde_derive::Deserialize;
use serenity::{
    framework::standard::{
        Args, CommandResult,
        macros::command,
    },
    model::channel::Message,
};
use serenity::prelude::*;
use log::*;
use chrono::{Utc, DateTime};
use chrono::prelude::*;

pub struct UserInfo {
    created_at: String,
    created_at_ago: i64,
    member: Option<MemberInfo>,
}

pub struct MemberInfo {
    nick: String,
    joined_at: String,
    joined_at_ago: String,
    roles: Vec<String>,
}

#[command]
#[description = "Display information about the user"]
#[only_in("guilds")]
pub fn userinfo(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let user = msg.mentions.get(0).ok_or("No user mentioned")?;

    let mut user_info = UserInfo {
        created_at: user.created_at().format("%d.%m.%Y %H:%M:%S").to_string(),
        created_at_ago: Utc::now()
            .signed_duration_since(user.created_at())
            .num_days(),
        member: None,
    };

    if let Some(guild_id) = msg.guild_id {
        let member = guild_id.member(&ctx, user.id)?;
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
                .and_then(|time| Some(time.format("%d.%m.%Y %H:%M:%S").to_string()))
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
                .and_then(|time| {
                    Some(
                        Utc::now()
                            .signed_duration_since(time)
                            .num_days()
                            .to_string(),
                    )
                })
                .unwrap_or_else(|| "Unknown".to_string())
        };
        let member_info = MemberInfo {
            nick: member
                .nick
                .unwrap_or_else(|| format!("{}#{}", user.name, user.discriminator)),
            joined_at,
            joined_at_ago,
            roles: member
                .roles
                .iter()
                .filter_map(|r| r.to_role_cached(&ctx.cache).and_then(|r| Some(r.name)))
                .collect(),
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
            .and_then(|m| Some(&m.joined_at))
            .unwrap_or(&default),
        user_info
            .member
            .as_ref()
            .and_then(|m| Some(&m.joined_at_ago))
            .unwrap_or(&default),
        user_info
            .member
            .as_ref()
            .and_then(|m| Some(m.roles.iter().map(|r| format!("{}, ", r)).collect::<String>()))
            .unwrap_or_else(|| default.clone()),
    );

    let _ = msg.channel_id.send_message(ctx, |m| {
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
    });
    Ok(())
}