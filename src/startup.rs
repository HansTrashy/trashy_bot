use crate::models::mute::Mute;
use crate::models::reminder::Reminder;
use crate::models::server_config::ServerConfig;
use crate::DatabasePool;
use chrono::Utc;
use serenity::model::{id::ChannelId, id::GuildId, id::RoleId, id::UserId};
use serenity::utils::MessageBuilder;
use tokio::time::sleep;
use tracing::error;

pub async fn init(client: &serenity::Client) {
    let pool = client
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .expect("Failed to get Pool")
        .clone();

    // restart reminders
    for r in Reminder::list(&pool).await.unwrap() {
        let http = client.cache_and_http.http.clone();
        let data = client.data.clone();
        if r.end_time <= Utc::now() {
            tokio::spawn(async move {
                let source_msg_id = r.source_msg_id;
                std::mem::drop(
                    ChannelId(r.channel_id as u64)
                        .send_message(&http, |m| {
                            m.content(
                                MessageBuilder::new()
                                    .push("Sorry, ")
                                    .mention(&UserId(r.user_id as u64))
                                    .push(" im late! You wanted me to remind you that: ")
                                    .push(r.msg)
                                    .build(),
                            )
                        })
                        .await,
                );

                let pool = data
                    .read()
                    .await
                    .get::<DatabasePool>()
                    .expect("Failed to get Pool")
                    .clone();

                std::mem::drop(Reminder::delete(&pool, source_msg_id).await);
            });
        } else {
            let duration = r.end_time.signed_duration_since(Utc::now());
            tokio::spawn(async move {
                sleep(duration.to_std().unwrap()).await;

                let source_msg_id = r.source_msg_id;
                std::mem::drop(
                    ChannelId(r.channel_id as u64)
                        .send_message(&http, |m| {
                            m.content(
                                MessageBuilder::new()
                                    .push("Hey, ")
                                    .mention(&UserId(r.user_id as u64))
                                    .push("! You wanted me to remind you that: ")
                                    .push(r.msg)
                                    .build(),
                            )
                        })
                        .await,
                );

                let pool = data
                    .read()
                    .await
                    .get::<DatabasePool>()
                    .expect("Failed to get Pool")
                    .clone();

                std::mem::drop(Reminder::delete(&pool, source_msg_id).await);
            });
        }
    }

    // restart unmute futures
    let server_configs = ServerConfig::list(&pool).await.unwrap();

    for m in Mute::list(&pool).await.unwrap() {
        let http = client.cache_and_http.http.clone();
        let data = client.data.clone();
        if let Some(config) = server_configs
            .iter()
            .filter(|x| x.server_id == m.server_id)
            .collect::<Vec<_>>()
            .first()
        {
            let config: crate::commands::config::Guild =
                serde_json::from_value(config.config.clone()).unwrap();

            let m_clone = m.clone();
            let remove_mute_fut = async move {
                match GuildId(m.server_id as u64)
                    .member(&http, m.user_id as u64)
                    .await
                {
                    Ok(mut member) => {
                        member
                            .remove_role(&http, RoleId(config.mute_role.unwrap()))
                            .await
                            .expect("Could not remove role");
                    }
                    Err(e) => error!("Could not get member: {:?}", e),
                };

                let pool = data
                    .read()
                    .await
                    .get::<DatabasePool>()
                    .expect("Failed to get Pool")
                    .clone();

                std::mem::drop(Mute::delete(&pool, m.server_id, m.user_id).await);
            };

            if m_clone.end_time <= Utc::now() {
                tokio::spawn(remove_mute_fut);
            } else {
                let duration = m_clone.end_time.signed_duration_since(Utc::now());
                let delayed_fut = async move {
                    sleep(duration.to_std().unwrap()).await;
                    remove_mute_fut.await;
                };
                tokio::spawn(delayed_fut);
            }
        }
    }
}

pub async fn init_xkcd(config: &crate::config::Config) {
    use crate::XKCD_INDEX;
    use crate::XKCD_INDEX_READER;
    use crate::XKCD_INDEX_SCHEMA;
    use tantivy::schema::*;

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("alt", TEXT | STORED);
    schema_builder.add_text_field("img", STORED);
    schema_builder.add_u64_field("number", IntOptions::default().set_stored().set_indexed());
    XKCD_INDEX_SCHEMA
        .set(schema_builder.build())
        .map_err(|_| "Could not init xkcd index schema")
        .unwrap();

    let schema = XKCD_INDEX_SCHEMA
        .get()
        .expect("XKCD index schema not initialized");

    let index = if std::path::Path::new(&config.xkcd_index).is_dir() {
        // Use existing index
        tantivy::Index::open_in_dir(&config.xkcd_index)
            .map_err(|e| format!("Failed to load tantivy index: {}", e))
            .expect("Could not load xkcd index")
    } else {
        // create index from scratch
        tokio::fs::create_dir_all(&config.xkcd_index)
            .await
            .expect("Could not create folder for xkcd index");
        tantivy::Index::create_in_dir(&config.xkcd_index, schema.clone())
            .map_err(|e| format!("Failed to create tantivy index: {}", e))
            .expect("Could not create xkcd index")
    };

    XKCD_INDEX.set(index).unwrap();
    let index = XKCD_INDEX.get().expect("XKCD index not initialized");

    let reader: tantivy::IndexReader = index
        .reader_builder()
        .reload_policy(tantivy::ReloadPolicy::OnCommit)
        .try_into()
        .expect("Failed to init index reader");

    XKCD_INDEX_READER
        .set(reader)
        .map_err(|_| "Could not init xkcd index reader")
        .unwrap();
}
