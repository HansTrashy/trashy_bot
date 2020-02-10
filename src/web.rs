use actix_web::{web, App, Error, HttpResponse, HttpServer};
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use futures::future::{ok, Future};
use log::*;
use serde_derive::Deserialize;
use serenity::model::id::{ChannelId, MessageId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Deserialize)]
pub struct Payload {
    data: Vec<StreamInfo>,
}

#[derive(Deserialize)]
pub struct StreamInfo {
    id: String,
    user_id: String,
    user_name: String,
    game_id: String,
    title: String,
    viewer_count: u64,
    started_at: String,
    language: String,
    thumbnail_url: String,
}

fn twitch_hook(
    pool: web::Data<Pool>,
    state: web::Data<Arc<State>>,
    twitch_user_id: web::Path<(String,)>,
    payload: web::Json<Payload>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    debug!("received webhook for: {}", twitch_user_id.0);

    let mut conn: &PgConnection = &pool.get().unwrap();

    if payload.data.is_empty() {
        // stream went offline -> update message
        state.offline_handler(&twitch_user_id.0);
    } else {
        // stream went online -> update message
        let message = format!("Twitch Stream: {} is online!", payload.data[0].user_name);
        state.online_handler(&twitch_user_id.0, &message);
    }

    ok(HttpResponse::Ok().finish())
}

pub struct State {
    // twitch id to (channel_id, message_id) for updates
    pub webhooks: Arc<RwLock<HashMap<String, (u64, u64)>>>,
}

impl State {
    fn new() -> Self {
        Self {
            webhooks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_webhook(&self, twitch_id: &str, channel_id: u64, message_id: u64) {
        let mut write = self.webhooks.write().unwrap();

        write.insert(twitch_id.to_owned(), (channel_id, message_id));
    }

    pub fn online_handler(&self, twitch_id: &str, message: &str) {
        let read = self.webhooks.read().unwrap();

        if let Some((channel_id, message_id)) = read.get(twitch_id) {
            let _ = ChannelId(*channel_id)
                .message(MessageId(*message_id))
                .and_then(|mut msg| msg.edit(|m| m.content(message)));
        }
    }

    pub fn offline_handler(&self, twitch_id: &str) {}
}

pub fn start(pool: Pool) -> Arc<State> {
    let state = Arc::new(State::new());
    let thread_copy = state.clone();
    thread::spawn(move || {
        HttpServer::new(move || {
            App::new()
                .data(pool.clone())
                .data(thread_copy.clone())
                .service(
                    web::resource("/webhooks/twitch/{twitch_user_id}")
                        .route(web::post().to_async(twitch_hook)),
                )
        })
        .bind("127.0.0.1:8080")?
        .run()
    });
    state
}
