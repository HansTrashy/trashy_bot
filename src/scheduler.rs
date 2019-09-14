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
    Dummy(Arc<Mutex<String>>),
}

impl Task {
    fn execute(&self, cache_and_http: Arc<CacheAndHttp>, db_pool: DbPool) {
            match &self {
                Self::Reply { user, channel, msg } => {
                    let _ = ChannelId(*channel).send_message(&*cache_and_http.http, |m| {
                        m.content(
                            MessageBuilder::new()
                                .mention(&UserId(*user))
                                .push(&msg)
                                .build(),
                        )
                    });
                }
                Self::RemoveMute { guild_id, user, mute_role } => {
                    use crate::schema::mutes;
                    use diesel::prelude::*;

                    let conn = db_pool.get().unwrap();

                    match GuildId(*guild_id).member(&*cache_and_http.http, UserId(*user)) {
                        Ok(mut member) => {
                            let _ = member.remove_role(&*cache_and_http.http, RoleId(*mute_role));
                        }
                        Err(e) => error!("could not get member: {:?}", e),
                    };

                    diesel::delete(
                        mutes::table
                            .filter(mutes::server_id.eq(*guild_id as i64))
                            .filter(mutes::user_id.eq(*user as i64)),
                    )
                    .execute(&*conn)
                    .expect("could not delete mute");
                },
                Self::Dummy(v) => {
                    let mut lock = v.lock().unwrap();
                    *lock = String::from("finished");
                }
            }
    }

    pub fn remove_mute(guild_id: u64, user: u64, mute_role: u64) -> Self {
        Self::RemoveMute { guild_id, user, mute_role }
    }

    pub fn reply(user: u64, channel: u64, msg: String) -> Self {
        Self::Reply { user, channel, msg }
    }

    pub fn dummy(v: Arc<Mutex<String>>) -> Self {
        Self::Dummy(v)
    }
}

type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

pub struct Scheduler {
    runtime: tokio::runtime::Runtime,
    cache_and_http: Arc<CacheAndHttp>,
    db_pool: DbPool,
}


impl Scheduler {
    pub fn new(cache_and_http: Arc<CacheAndHttp>, db_pool: DbPool) -> Self {
        Self {
            runtime: tokio::runtime::Runtime::new().unwrap(),
            cache_and_http,
            db_pool,
        }
    }

    pub fn add_task(&self, duration: Duration, task: Task) {
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

#[cfg(test)]
mod tests {
    use std::thread;
    use time::Duration;
    use super::{Task, Scheduler};
    use std::sync::{Arc, Mutex};
    use serenity::CacheAndHttp;
    use super::DbPool;

    // #[test]
    // fn test_scheduler() {
    //     let scheduler = Arc::new(Scheduler::new(Arc::new(CacheAndHttp::default()), DbPool::new()));
    //     let v = Arc::new(Mutex::new(String::from("init")));
    //     let v2 = Arc::new(Mutex::new(String::from("init2")));

    //     let task = Task::dummy(Arc::clone(&v));
    //     let task2 = Task::dummy(Arc::clone(&v2));

    //     scheduler.add_task(Duration::milliseconds(200), task);
    //     scheduler.add_task(Duration::milliseconds(400), task2);

    //     assert_eq!("init", &*v.lock().unwrap());
    //     assert_eq!("init2", &*v2.lock().unwrap());

    //     thread::sleep(Duration::milliseconds(300).to_std().unwrap());

    //     assert_eq!("finished", &*v.lock().unwrap());
    //     assert_eq!("init2", &*v2.lock().unwrap());

    //     thread::sleep(Duration::milliseconds(200).to_std().unwrap());

    //     assert_eq!("finished", &*v.lock().unwrap());
    //     assert_eq!("finished", &*v2.lock().unwrap());
    // }
}
