use chrono::{DateTime, Utc};
use log::*;
use serde::{Deserialize, Serialize};
use serenity::utils::MessageBuilder;
use serenity::CacheAndHttp;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
    model::id::{ChannelId, GuildId, RoleId, UserId},
};
use std::sync::{Arc, Mutex};
use time::Duration;
use tokio::time::delay_for;

#[derive(PartialEq, Eq, Serialize, Deserialize, Clone)]
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
                            .push(" ")
                            .push(&msg)
                            .build(),
                    )
                });
            }
            Self::RemoveMute {
                guild_id,
                user,
                mute_role,
            } => {
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
            }
        }
    }

    pub fn remove_mute(guild_id: u64, user: u64, mute_role: u64) -> Self {
        Self::RemoveMute {
            guild_id,
            user,
            mute_role,
        }
    }

    pub fn reply(user: u64, channel: u64, msg: String) -> Self {
        Self::Reply { user, channel, msg }
    }
}

type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub struct Scheduler {
    runtime: Arc<tokio::runtime::Runtime>,
    cache_and_http: Arc<CacheAndHttp>,
    db_pool: DbPool,
    task_list: Arc<Mutex<Vec<(DateTime<Utc>, Task)>>>,
}

impl Scheduler {
    pub fn new(
        rt: Arc<tokio::runtime::Runtime>,
        cache_and_http: Arc<CacheAndHttp>,
        db_pool: DbPool,
    ) -> Self {
        let task_list = Arc::new(Mutex::new(Vec::new()));

        if let Ok(data) = std::fs::read_to_string("scheduler_state.storage") {
            let when_tasks =
                serde_json::from_str::<Vec<(DateTime<Utc>, Task)>>(&data).unwrap_or_default();
            for (when, task) in when_tasks {
                let duration_until = when.signed_duration_since(chrono::Utc::now());
                let cache_and_http_clone = cache_and_http.clone();
                let db_pool_clone = db_pool.clone();
                let task_list_clone = Arc::clone(&task_list);
                if duration_until > Duration::zero() {
                    task_list.lock().unwrap().push((when, task.clone()));
                    let f = async move {
                        delay_for(duration_until.to_std().unwrap()).await;
                        task_list_clone.lock().unwrap().retain(|(_, t)| t != &task);
                        task.execute(cache_and_http_clone, db_pool_clone);
                    };
                    rt.spawn(f);
                }
            }
        }

        Self {
            runtime: rt,
            cache_and_http,
            db_pool,
            task_list,
        }
    }

    pub fn add_task(&self, duration: Duration, task: Task) {
        {
            let when = chrono::Utc::now() + duration;
            let mut lock = self.task_list.lock().unwrap();
            lock.push((when, task.clone()));
            let data = serde_json::to_string(&*lock).expect("could not serialize rules state");
            std::fs::write("scheduler_state.storage", data)
                .expect("could not write rules state to file");
        }

        let cache_and_http = Arc::clone(&self.cache_and_http);
        let db_pool = self.db_pool.clone();
        let task_list_clone = Arc::clone(&self.task_list);
        let f = async move {
            delay_for(duration.to_std().unwrap()).await;
            task_list_clone.lock().unwrap().retain(|(_, t)| t != &task);
            task.execute(cache_and_http, db_pool);
        };
        self.runtime.spawn(f);
    }
}
