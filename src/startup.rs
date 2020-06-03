use crate::models::mute::Mute;
use crate::models::reminder::Reminder;
use crate::models::server_config::ServerConfig;
use crate::DatabasePool;
use chrono::Utc;
use serenity::model::{id::ChannelId, id::GuildId, id::RoleId, id::UserId};
use serenity::utils::MessageBuilder;
use tokio::time::delay_for;
use tracing::error;

async fn on_startup(client: &serenity::Client) {
    let mut db_client = client
        .data
        .read()
        .await
        .get::<DatabasePool>()
        .expect("Failed to get Pool")
        .get()
        .await
        .expect("no connection for startup available");

    // restart reminders
    for r in Reminder::list(&mut db_client).await.unwrap() {
        let http = client.cache_and_http.http.clone();
        let data = client.data.clone();
        if r.end_time <= Utc::now() {
            tokio::spawn(async move {
                let source_msg_id = r.source_msg_id;
                let _ = ChannelId(r.channel_id as u64)
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
                    .await;

                let mut db_client = data
                    .read()
                    .await
                    .get::<DatabasePool>()
                    .expect("Failed to get Pool")
                    .get()
                    .await
                    .expect("no connection for startup available");

                let _ = Reminder::delete(&mut db_client, source_msg_id).await;
            });
        } else {
            let duration = r.end_time.signed_duration_since(Utc::now());
            tokio::spawn(async move {
                delay_for(duration.to_std().unwrap()).await;

                let source_msg_id = r.source_msg_id;
                let _ = ChannelId(r.channel_id as u64)
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
                    .await;

                let mut db_client = data
                    .read()
                    .await
                    .get::<DatabasePool>()
                    .expect("Failed to get Pool")
                    .get()
                    .await
                    .expect("no connection for startup available");

                let _ = Reminder::delete(&mut db_client, source_msg_id).await;
            });
        }
    }

    // restart unmute futures
    let server_configs = ServerConfig::list(&mut db_client).await.unwrap();

    for m in Mute::list(&mut db_client).await.unwrap() {
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
                            .expect("could not remove role");
                    }
                    Err(e) => error!("could not get member: {:?}", e),
                };

                let mut db_client = data
                    .read()
                    .await
                    .get::<DatabasePool>()
                    .expect("Failed to get Pool")
                    .get()
                    .await
                    .expect("no connection for startup available");

                let _ = Mute::delete(&mut db_client, m.server_id, m.user_id).await;
            };

            if m_clone.end_time <= Utc::now() {
                tokio::spawn(remove_mute_fut);
            } else {
                let duration = m_clone.end_time.signed_duration_since(Utc::now());
                let delayed_fut = async move {
                    delay_for(duration.to_std().unwrap()).await;
                    remove_mute_fut.await;
                };
                tokio::spawn(delayed_fut);
            }
        }
    }
}
