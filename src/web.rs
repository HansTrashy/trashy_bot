use actix_web::{web, App, Error, HttpResponse, HttpServer};
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use futures::future::{ok, Future};
use log::*;
use std::thread;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

fn twitch_hook(
    pool: web::Data<Pool>,
    twitch_user_id: web::Path<String>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    debug!("received webhook for: {}", twitch_user_id);

    let conn: &PgConnection = &pool.get().unwrap();

    ok(HttpResponse::Ok().finish())
}

pub fn start(pool: Pool) {
    thread::spawn(move || {
        HttpServer::new(move || {
            App::new().data(pool.clone()).service(
                web::resource("/webhooks/twitch/{twitch_user_id}")
                    .route(web::post().to_async(twitch_hook)),
            )
        })
        .bind("127.0.0.1:8080")?
        .run()
    });
}
