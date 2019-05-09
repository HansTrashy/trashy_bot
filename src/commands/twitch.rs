use log::*;
use serenity::model::channel;

use serde_derive::*;

extern crate reqwest;

use reqwest::Error;
use std::time::Duration;
use reqwest::ClientBuilder;
use std::io::Read;
use std::{env};
//use http::{Request, Response};

command!(ping(_ctx, msg, args) {
        info!("In Twitch Command Ping");
        msg.channel_id.say("Twitch!");
    });

command!(add_user(_ctx, msg, args) {
        let discord_user = args.single::<String>().unwrap();
        let twitch_user_name = args.single::<String>().unwrap();
        let twitch_user_response = get_twitch_user_by_name(twitch_user_name);
        let twitch_user_json: serde_json::Value = serde_json::from_str(twitch_user_response)?;
        info!("Test ID: {}", twitch_user_json["data"]["id"]);
        //let twitch_user_json: Twitch_User = serde_json::from_str(&buf).unwrap();
        //let twitch_user_json_data: data = serde_json::from_str(&twitch_user_response.Result).unwrap();
        //info!("User id: {:#?}", twitch_user_response[0].unwrap());
    });


#[derive(Deserialize)]
struct Twitch_User {
    data: Vec<serde_json::Value>,
}

fn get_twitch_user_by_name(twitch_user_name: String) -> Result<(String), Error>{
    let token = env::var("TWITCH_TOKEN").expect("Expected a token in the environment");

    let request_url = "https://api.twitch.tv/helix/users?login=theneriik";
    let client = reqwest::Client::new();
    let mut resp = client.get(request_url)
        .header("Accept-Charset", "utf-8")
        .header("Accept", "application/vnd.twitchtv.v5+json")
        .header("Client-ID", token)
        .send()?;

    let mut buf = String::new();
    resp.read_to_string(&mut buf).expect("Failed to read response");
    Ok(buf)
}
