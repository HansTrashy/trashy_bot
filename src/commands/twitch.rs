use log::*;
use serde_derive::*;
use crate::DatabaseConnection;
use std::env;

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

command!(add_stream(ctx, msg, args) {
    let data = ctx.data.lock();
    let conn = match data.get::<DatabaseConnection>() {
        Some(v) => v.get().unwrap(),
        None => {
            let _ = msg.reply("Could not retrieve the database connection!");
            return Ok(());
        }
    };

    let twitch_user_name = args.single::<String>().unwrap();
    let twitch_result = get_twitch_user_by_name(&twitch_user_name);

    // Step 1: Check if the guild config allows a stream

    // Step 2: Check if the stream already exits, if not create it

    // Step 3: Check if a sub already exists, if not create it

    // use this stuff in the appropiate places above ^
    let twitch_result = match twitch_result {
        Ok(result) => result,
        Err(e) => {
            error!("Could not retrive data from Twitch: {:?}", e);
            let _ = msg.channel_id.say("Da lief etwas mit der Twitch API schief, versuche es später noch einmal!");
            return Ok(())
        }
    };

    let channel_link = format!("https://www.twitch.tv/{}", &twitch_result.data[0].display_name);

    // Placeholder message could be made more pretty
    let _ = msg.channel_id.send_message(|m| m.embed(|e| 
        e.author(|a| a.name("Twitch User Info"))
        .title(&twitch_user_name)
        .description(&twitch_result.data[0].description)
        .color((0,120,220))
        .image(&twitch_result.data[0].profile_image_url)
        .url(channel_link)));
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