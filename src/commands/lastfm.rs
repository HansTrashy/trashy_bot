use crate::models::lastfm::Lastfm;
use crate::util::{get_client, get_reqwest_client, timed_request};
use serenity::prelude::*;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use tracing::info;

#[command]
#[description = "Link your lastfm account to your discord account"]
#[example = "HansTrashy"]
#[usage = "*lastfm_username*"]
#[num_args(1)]
pub async fn register(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let username = args.single::<String>()?;
    let pool = get_client(&ctx).await?;

    if let Ok(user) = Lastfm::get(&pool, *msg.author.id.as_u64() as i64).await {
        let lastfm = Lastfm::update(&pool, user.id, username).await?;

        msg.reply(
            ctx,
            format!("Updated your lastfm username to {}", lastfm.username),
        )
        .await?;
    } else {
        let lastfm = Lastfm::create(&pool, *msg.author.id.as_u64() as i64, username).await?;

        msg.reply(
            ctx,
            format!("Added {} as your lastfm username!", lastfm.username),
        )
        .await?;
    }

    Ok(())
}

#[command]
#[description = "Show your currently playing track"]
#[num_args(0)]
#[bucket = "lastfm"]
pub async fn now(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let pool = get_client(&ctx).await?;
    let lastfm = Lastfm::get(&pool, *msg.author.id.as_u64() as i64).await?;
    let lastfm_api_key = ctx
        .data
        .read()
        .await
        .get::<crate::Config>()
        .ok_or("Failed to access config")?
        .lastfm_api_key
        .clone();

    // prepare for the lastfm api
    let url = format!("http://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user={}&api_key={}&format=json",
            lastfm.username,
            &lastfm_api_key);

    let (res, request_time) = timed_request(&get_reqwest_client(&ctx).await?, &url).await?;

    // ignore the case where users only played a single title and there is no array
    if let Some(tracks) = res
        .pointer("/recenttracks/track")
        .and_then(|a| a.as_array())
    {
        for t in tracks {
            // here we have a boolean that only ever can be true, otherwise it is non existent, also, it is a string
            if t.pointer("/@attr/nowplaying")
                .and_then(|a| a.as_str())
                .unwrap_or("")
                == "true"
            {
                let content = format!(
                    "Artist: {} - {}",
                    t.pointer("/artist/#text")
                        .and_then(|a| a.as_str())
                        .unwrap_or("Unknown Artist"),
                    t.pointer("/name")
                        .and_then(|a| a.as_str())
                        .unwrap_or("Unknown Title")
                );

                msg.channel_id
                    .send_message(&ctx, |m| {
                        m.embed(|e| {
                            e.description(&content).footer(|f| {
                                f.text(format!(
                                    "Lastfm response took: {}ms",
                                    request_time.as_millis()
                                ))
                            })
                        })
                    })
                    .await?;
            }
        }
    }

    Ok(())
}

#[command]
#[description = "Show your recent tracks"]
#[num_args(0)]
#[bucket = "lastfm"]
pub async fn recent(ctx: &Context, msg: &Message, _args: Args) -> CommandResult {
    let pool = get_client(&ctx).await?;
    let lastfm = Lastfm::get(&pool, *msg.author.id.as_u64() as i64).await?;
    let lastfm_api_key = ctx
        .data
        .read()
        .await
        .get::<crate::Config>()
        .ok_or("Failed to access config")?
        .lastfm_api_key
        .clone();

    // prepare for the lastfm api
    let url = format!("http://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user={}&api_key={}&format=json&limit=10",
            lastfm.username,
            &lastfm_api_key);

    let (res, request_time) = timed_request(&get_reqwest_client(&ctx).await?, &url).await?;

    let mut content = String::new();

    // ignore the case where users only played a single title and there is no array
    if let Some(tracks) = res
        .pointer("/recenttracks/track")
        .and_then(|a| a.as_array())
    {
        for t in tracks {
            content.push_str(&format!(
                "Artist: {} - {}\n",
                t.pointer("/artist/#text")
                    .and_then(|a| a.as_str())
                    .unwrap_or("Unknown Artist"),
                t.pointer("/name")
                    .and_then(|a| a.as_str())
                    .unwrap_or("Unknown Title"),
            ));
        }
    }

    msg.channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.description(&content).footer(|f| {
                    f.text(format!(
                        "Lastfm response took: {}ms",
                        request_time.as_millis()
                    ))
                })
            })
        })
        .await?;

    Ok(())
}

#[command]
#[description = "Show your top artists"]
#[usage = "(all|7d|1m|3m|6m|12m)"]
#[example = "3m"]
#[min_args(0)]
#[max_args(1)]
#[bucket = "lastfm"]
pub async fn artists(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let period = match args.rest() {
        "all" => "overall",
        "7d" => "7day",
        "1m" => "1month",
        "3m" => "3month",
        "6m" => "6month",
        "12m" => "12month",
        _ => "overall",
    };
    let pool = get_client(&ctx).await?;
    let lastfm_api_key = ctx
        .data
        .read()
        .await
        .get::<crate::Config>()
        .ok_or("Failed to access config")?
        .lastfm_api_key
        .clone();

    let lastfm = Lastfm::get(&pool, *msg.author.id.as_u64() as i64).await?;

    // prepare for the lastfm api
    let url = format!("http://ws.audioscrobbler.com/2.0/?method=user.gettopartists&user={}&api_key={}&format=json&limit=10&period={}",
            lastfm.username,
            &lastfm_api_key,
            period);

    let (res, request_time) = timed_request(&get_reqwest_client(&ctx).await?, &url).await?;

    let mut content = String::new();

    if let Some(artists) = res.pointer("/topartists/artist").and_then(|a| a.as_array()) {
        for a in artists {
            content.push_str(&format!(
                "Rank: {} | {}\n",
                a.pointer("/@attr/rank")
                    .and_then(|a| a.as_str())
                    .unwrap_or("Unknown Rank"),
                a.pointer("/name")
                    .and_then(|a| a.as_str())
                    .unwrap_or("Unknown Artist"),
            ));
        }
    }

    msg.channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.description(&content).footer(|f| {
                    f.text(format!(
                        "Lastfm response took: {}ms",
                        request_time.as_millis()
                    ))
                })
            })
        })
        .await?;

    Ok(())
}

#[command]
#[description = "Show your top albums"]
#[usage = "(all|7d|1m|3m|6m|12m)"]
#[example = "3m"]
#[min_args(0)]
#[max_args(1)]
#[bucket = "lastfm"]
pub async fn albums(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let period = match args.rest() {
        "all" => "overall",
        "7d" => "7day",
        "1m" => "1month",
        "3m" => "3month",
        "6m" => "6month",
        "12m" => "12month",
        _ => "overall",
    };
    let pool = get_client(&ctx).await?;
    let lastfm_api_key = ctx
        .data
        .read()
        .await
        .get::<crate::Config>()
        .ok_or("Failed to access config")?
        .lastfm_api_key
        .clone();

    let lastfm = Lastfm::get(&pool, *msg.author.id.as_u64() as i64).await?;

    // prepare for the lastfm api
    let url = format!("http://ws.audioscrobbler.com/2.0/?method=user.gettopalbums&user={}&api_key={}&format=json&limit=10&period={}",
            lastfm.username,
            &lastfm_api_key,
            period);

    let (res, request_time) = timed_request(&get_reqwest_client(&ctx).await?, &url).await?;

    let mut content = String::new();

    if let Some(albums) = res.pointer("/topalbums/album").and_then(|a| a.as_array()) {
        for a in albums {
            content.push_str(&format!(
                "Rank: {} | {}\n",
                a.pointer("/@attr/rank")
                    .and_then(|a| a.as_str())
                    .unwrap_or("Unknown Rank"),
                a.pointer("/name")
                    .and_then(|a| a.as_str())
                    .unwrap_or("Unknown Artist"),
            ));
        }
    }

    msg.channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.description(&content).footer(|f| {
                    f.text(format!(
                        "Lastfm response took: {}ms",
                        request_time.as_millis()
                    ))
                })
            })
        })
        .await?;

    Ok(())
}

#[command]
#[description = "Show your top tracks"]
#[usage = "(all|7d|1m|3m|6m|12m)"]
#[example = "3m"]
#[min_args(0)]
#[max_args(1)]
#[bucket = "lastfm"]
pub async fn tracks(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let period = match args.rest() {
        "all" => "overall",
        "7d" => "7day",
        "1m" => "1month",
        "3m" => "3month",
        "6m" => "6month",
        "12m" => "12month",
        _ => "overall",
    };
    let pool = get_client(&ctx).await?;
    let lastfm_api_key = ctx
        .data
        .read()
        .await
        .get::<crate::Config>()
        .ok_or("Failed to access config")?
        .lastfm_api_key
        .clone();

    info!("period: {:?}", period);

    let lastfm = Lastfm::get(&pool, *msg.author.id.as_u64() as i64).await?;

    // prepare for the lastfm api
    let url = format!("http://ws.audioscrobbler.com/2.0/?method=user.gettoptracks&user={}&api_key={}&format=json&limit=10&period={}",
            lastfm.username,
            &lastfm_api_key,
            period);

    let (res, request_time) = timed_request(&get_reqwest_client(&ctx).await?, &url).await?;

    let mut content = String::new();

    if let Some(tracks) = res.pointer("/toptracks/track").and_then(|a| a.as_array()) {
        let mut overall = 0;
        for t in tracks {
            let playcount = t.pointer("/playcount").and_then(|a| a.as_str());

            overall += playcount
                .map(|x| x.parse::<i32>().unwrap_or(0))
                .unwrap_or(0);
            content.push_str(&format!(
                "Rank: {} | Played: {} | {} - {}\n",
                t.pointer("/@attr/rank")
                    .and_then(|a| a.as_str())
                    .unwrap_or("Unknown Rank"),
                playcount.unwrap_or("-"),
                t.pointer("/artist/name")
                    .and_then(|a| a.as_str())
                    .unwrap_or("Unknown Artist"),
                t.pointer("/name")
                    .and_then(|a| a.as_str())
                    .unwrap_or("Unknown Track"),
            ));
        }
        content.push_str(&format!("Overall scrobbles: {}\n", overall));
    }

    msg.channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.description(&content).footer(|f| {
                    f.text(format!(
                        "Lastfm response took: {}ms",
                        request_time.as_millis()
                    ))
                })
            })
        })
        .await?;

    Ok(())
}
