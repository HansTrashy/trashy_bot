use log::*;
use serde_derive::*;
use crate::DatabaseConnection;
use std::env;
use crate::models::twitch_config::TwitchConfig;
use crate::models::twitch_stream::TwitchStream;
use crate::models::twitch_sub::TwitchSub;

command!(add_stream(ctx, msg, args) {
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };

    let twitch_user_id = args.single::<String>().unwrap();
    
    let guild_id = *msg.channel_id.to_channel().expect("not a guild channel").guild().unwrap().read().guild_id.as_u64() as i64;

    // Step 1: Check if the guild config allows a stream in this channel
    {
        use crate::schema::twitch_configs::dsl;
        use diesel::prelude::*;

        let guild_config = dsl::twitch_configs.filter(dsl::guild_id.eq(guild_id)).first::<TwitchConfig>(&conn).unwrap();
        if !guild_config.channel_ids.contains(&(*msg.channel_id.as_u64() as i64)) {
            let _ = msg.reply("TwitchConfig ist nicht eingerichtet für diesen Server!");
            return Ok(());
        }
    };

    // Step 2: Check if the stream already exits, if not create it
    let stream = {
        use crate::schema::twitch_streams::dsl;
        use diesel::prelude::*;

        let twitch_stream_result = dsl::twitch_streams.filter(dsl::twitch_user_id.eq(&twitch_user_id)).first::<TwitchStream>(&conn);
        match twitch_stream_result {
            Ok(stream) => stream,
            Err(_e) => {
                let twitch_result = get_twitch_user_by_name(&twitch_user_id).unwrap();
                crate::models::twitch_stream::create_twitch_stream(&conn, twitch_user_id, twitch_result.data[0].profile_image_url.to_owned())
            }
        }
    };

    // Step 3: Check if a sub already exists, if not create it
    let sub = {
        use crate::schema::twitch_subs::dsl;
        use diesel::prelude::*;

        let twitch_sub_result = dsl::twitch_subs.filter(dsl::twitch_stream_id.eq(stream.id)).first::<TwitchSub>(&conn);
        match twitch_sub_result {
            Ok(sub) => sub,
            Err(_e) => {
                crate::models::twitch_sub::create_twitch_sub(&conn, stream.id, *msg.channel_id.as_u64() as i64, *msg.author.id.as_u64() as i64, None)
            }
        }
    };

    //TODO: Should probably create a sub in the system thats TBD so a webhook endpoint is listening for updates
});

/// Calls the Twitch API and returns a TwitchResult JSON containing the User data
fn get_twitch_user_by_name(twitch_user_name: &str) -> Result<TwitchResult, reqwest::Error> {
    let token = env::var("TWITCH_TOKEN").expect("Expected a token in the environment");

    let request_url = format!("https://api.twitch.tv/helix/users?login={}", &twitch_user_name);
    let client = reqwest::Client::new();
    Ok(client.get(&request_url)
        .header("Accept-Charset", "utf-8")
        .header("Accept", "application/vnd.twitchtv.v5+json")
        .header("Client-ID", token)
        .send()?
        .json::<TwitchResult>()?)
}

#[derive(Deserialize)]
struct TwitchResult {
    data: Vec<User>,
}

/// Twitch Endpoint: https://dev.twitch.tv/docs/api/reference/#get-users
/// Note: E-Mail field in response is missing in our struct because the API doesn't return it when calling by Client-ID
#[derive(Deserialize)]
struct User {
    id: String,
    login: String,
    display_name: String,
    #[serde(rename = "type")]
    kind: String,
    broadcaster_type: String,
    description: String,
    profile_image_url: String,
    offline_image_url: String,
    view_count: u64,
}

fn setup_webhook_with_twitch_id(twitch_user_id: String) -> Result<PayloadWebhookSetup, reqwest::Error> {
    let token = env::var("TWITCH_TOKEN").expect("Expected a TWITCH_TOKEN in the ENV vars");

    let topic_url = format!("https://api.twitch.tv/helix/streams?user_id={}", &twitch_user_id);
    let request_url = format!("https://api.twitch.tv/helix/webhooks/hub");
    let client = reqwest::Client::new();
    Ok(client.post(&request_url).json(&SubscribeRequest::default()).header("Accept-Charset", "utf-8")
        .header("Accept", "application/vnd.twitchtv.v5+json")
        .header("Client-ID", token)
        .send()?
        .json::<PayloadWebhookSetup>()?)
}


#[derive(Serialize, Default)]
pub struct SubscribeRequest {
    #[serde(rename = "hub.callback")]
    hub_callback: String,
    #[serde(rename = "hub.mode")]
    hub_mode: String,
    #[serde(rename = "hub.topic")] 
    hub_topic: String, // https://api.twitch.tv/helix/streams
    #[serde(rename = "hub.lease_seconds")]
    hub_lease_seconds: u64, // use 0 to test subscription workflow
    #[serde(rename = "hub.secret")]
    hub_secret: String, // used to verify that this subscription was done by the bot itself
}

#[derive(Deserialize)]
pub struct PayloadWebhookSetup {
    data: Vec<WebhookResponse>,
}

#[derive(Deserialize)]
pub struct WebhookResponse {
    // ??
}