use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};
use time::Duration;
use serenity::CacheAndHttp;
use serenity::utils::MessageBuilder;
use log::*;
use serenity::{
    framework::standard::{Args, CommandResult, macros::command},
    model::channel::Message,
    model::id::{RoleId, ChannelId, UserId, GuildId},
};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum Task {
    Reply {
        user: u64,
        channel: u64,
        msg: String,
    },
    RemoveMute {
        guild_id: u64,
        user: u64,
        mute_role: u64,
    },
}

impl Task {
    fn execute(self, cache_and_http: Arc<CacheAndHttp>, db_pool: DbPool) {
            match self {
                Self::Reply { user, channel, msg } => {
                    let _ = ChannelId(channel).send_message(&*cache_and_http.http, |m| {
                        m.content(
                            MessageBuilder::new()
                                .mention(&UserId(user))
                                .push(&msg)
                                .build(),
                        )
                    });
                }
                Self::RemoveMute { guild_id, user, mute_role } => {
                    use crate::schema::mutes;
                    use diesel::prelude::*;

                    let conn = db_pool.get().unwrap();

                    match GuildId(guild_id).member(&*cache_and_http.http, UserId(user)) {
                        Ok(mut member) => {
                            let _ = member.remove_role(&*cache_and_http.http, RoleId(mute_role));
                        }
                        Err(e) => error!("could not get member: {:?}", e),
                    };

                    diesel::delete(
                        mutes::table
                            .filter(mutes::server_id.eq(guild_id as i64))
                            .filter(mutes::user_id.eq(user as i64)),
                    )
                    .execute(&*conn)
                    .expect("could not delete mute");
                },
            }
    }

    pub fn remove_mute(guild_id: u64, user: u64, mute_role: u64) -> Self {
        Self::RemoveMute { guild_id, user, mute_role }
    }

    pub fn reply(user: u64, channel: u64, msg: String) -> Self {
        Self::Reply { user, channel, msg }
    }
}

type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub struct Scheduler {
    runtime: tokio::runtime::Runtime,
    cache_and_http: Arc<CacheAndHttp>,
    db_pool: DbPool,
    task_list: Mutex<Vec<Task>>,
}


impl Scheduler {
    pub fn new(cache_and_http: Arc<CacheAndHttp>, db_pool: DbPool) -> Self {
        //TODO: load tasks from file/db
        Self {
            runtime: tokio::runtime::Runtime::new().unwrap(),
            cache_and_http,
            db_pool,
            task_list: Mutex::new(Vec::new()),
        }
    }

    pub fn add_task(&self, duration: Duration, task: Task) {
        self.task_list.lock().unwrap().push(task.clone());

        let cache_and_http = Arc::clone(&self.cache_and_http);
        let db_pool = self.db_pool.clone();
        let f = async move {
            let when = tokio::clock::now() + duration.to_std().unwrap();
            tokio::timer::delay(when).await;
            task.execute(cache_and_http, db_pool);
        };
        self.runtime.spawn(f);
    }
}

