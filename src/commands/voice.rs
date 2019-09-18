use serenity::{
    framework::standard::{Args, CommandResult, macros::command},
    model::channel::Message,
    client::bridge::voice::ClientVoiceManager,
    client::Context,
    voice,
};
use serenity::prelude::*;
use log::*;
use crate::VoiceManager;

#[command]
fn deafen(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            msg.channel_id.say(&ctx.http, "Works only on guilds")?;
            return Ok(());
        }
    };

    let manager_lock = ctx.data.read().get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock();

    let handler = match manager.get_mut(guild_id) {
        Some(handler) => handler,
        None => {
            msg.reply(&ctx, "Not in a voice channel")?;
            return Ok(());
        }
    };

    if handler.self_deaf {
        msg.channel_id.say(&ctx.http, "Already deafened")?;
    } else {
        handler.deafen(true);
        msg.channel_id.say(&ctx.http, "Deafened")?;
    }
    Ok(())
}

#[command]
fn join(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild = match msg.guild(&ctx.cache) {
        Some(guild) => guild,
        None => {
            msg.channel_id.say(&ctx.http, "Works only on guilds")?;
            return Ok(());
        }
    };

    let guild_id = guild.read().id;

    let channel_id = guild
        .read()
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg.reply(&ctx, "Not in a voice channel")?;
            return Ok(());
        }
    };

    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    if manager.join(guild_id, connect_to).is_some() {
        msg.channel_id
            .say(&ctx.http, &format!("Joined {}", connect_to.mention()))?;
    } else {
        msg.channel_id.say(&ctx.http, "Error joining the channel")?;
    }

    Ok(())
}

#[command]
fn leave(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            msg.channel_id.say(&ctx.http, "Works only on guilds")?;
            return Ok(());
        }
    };

    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        manager.remove(guild_id);
        msg.channel_id.say(&ctx.http, "Left voice channel")?;
    } else {
        msg.reply(&ctx, "Not in a voice channel")?;
    }

    Ok(())
}

#[command]
fn mute(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            msg.channel_id
                .say(&ctx.http, "Groups and DMs not supported")?;
            return Ok(());
        }
    };

    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    let handler = match manager.get_mut(guild_id) {
        Some(handler) => handler,
        None => {
            msg.reply(&ctx, "Not in a voice channel")?;
            return Ok(());
        }
    };

    if handler.self_mute {
        msg.channel_id.say(&ctx.http, "Already muted")?;
    } else {
        handler.mute(true);
        msg.channel_id.say(&ctx.http, "Now muted")?;
    }

    Ok(())
}

#[command]
fn play(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            msg.channel_id
                .say(&ctx.http, "Must provide a URL to a video or audio")?;
            return Ok(());
        }
    };

    if !url.starts_with("http") {
        msg.channel_id.say(&ctx.http, "Must provide a valid URL")?;
        return Ok(());
    }

    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            msg.channel_id
                .say(&ctx.http, "Error finding channel info")?;
            return Ok(());
        }
    };

    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        let source = match voice::ytdl(&url) {
            Ok(source) => source,
            Err(why) => {
                error!("Err starting source: {:?}", why);
                msg.channel_id.say(&ctx.http, "Error sourcing ffmpeg")?;
                return Ok(());
            }
        };

        handler.play_only(source);

        msg.channel_id.say(&ctx.http, "Playing song")?;
    } else {
        msg.channel_id
            .say(&ctx.http, "Not in a voice channel to play in")?;
    }

    Ok(())
}

#[command]
fn undeafen(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            msg.channel_id
                .say(&ctx.http, "Error finding channel info")?;
            return Ok(());
        }
    };

    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        handler.deafen(false);

        msg.channel_id.say(&ctx.http, "Undeafened")?;
    } else {
        msg.channel_id
            .say(&ctx.http, "Not in a voice channel to undeafen in")?;
    }

    Ok(())
}

#[command]
fn unmute(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match ctx.cache.read().guild_channel(msg.channel_id) {
        Some(channel) => channel.read().guild_id,
        None => {
            msg.channel_id
                .say(&ctx.http, "Error finding channel info")?;
            return Ok(());
        }
    };
    let manager_lock = ctx
        .data
        .read()
        .get::<VoiceManager>()
        .cloned()
        .expect("Expected VoiceManager in ShareMap.");
    let mut manager = manager_lock.lock();

    if let Some(handler) = manager.get_mut(guild_id) {
        handler.mute(false);
        msg.channel_id.say(&ctx.http, "Unmuted")?;
    } else {
        msg.channel_id
            .say(&ctx.http, "Not in a voice channel to undeafen in")?;
    }

    Ok(())
}
