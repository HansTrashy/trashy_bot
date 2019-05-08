use log::*;
use serenity::model::channel;
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
        info!("In Twitch Command adduser");
        let twitch_user_name = args.single::<String>().unwrap();
        get_twitch_user_by_name();
        msg.channel_id.say(&format!("Adding Twitch user {}!", &twitch_user_name));
    });



fn get_twitch_user_by_name() -> Result<(), Error>{
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
    info!("{}", buf);
    Ok(())
}
