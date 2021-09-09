#![deny(clippy::all)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_wrap)]
#![deny(nonstandard_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]
#![warn(unused)]
// TODO: remove this when sqlx fixed the macro calls with `_expr`
#![allow(clippy::used_underscore_binding)]
//! Trashy Bot

use anyhow::{Context, Result};
use trashy_bot::{config::Config, TrashyBot};

#[tokio::main]
async fn main() -> Result<()> {
    let config: Config = toml::from_str(
        &tokio::fs::read_to_string("config.toml")
            .await
            .context("Could not load config file")?,
    )
    .context("Failed to parse config")?;

    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    TrashyBot::run(config).await.context("failed to run bot")?;

    Ok(())

    // let http = Http::new_with_token(&config.discord_token);

    // let (owners, bot_id) = match http.get_current_application_info().await {
    //     Ok(info) => {
    //         let mut owners = HashSet::new();
    //         owners.insert(info.owner.id);

    //         (owners, info.id)
    //     }
    //     Err(why) => panic!("Could not access application info: {:?}", why),
    // };

    // let mut framework = StandardFramework::new()
    //     .configure(|c| {
    //         c.with_whitespace(true)
    //             .on_mention(Some(bot_id))
    //             .prefix("$")
    //             .delimiter(' ')
    //             .owners(owners)
    //     })
    //     .before(before)
    //     .after(after)
    //     .unrecognised_command(unknown_command)
    //     .normal_message(normal_message)
    //     .on_dispatch_error(dispatch_error)
    //     .help(&MY_HELP);

    // for bucket in &config.buckets {
    //     framework = framework
    //         .bucket(&bucket.name, |mut b| {
    //             if let Some(delay) = bucket.delay {
    //                 b = b.delay(delay);
    //             }
    //             if let Some(time_span) = bucket.time_span {
    //                 b = b.time_span(time_span);
    //             }
    //             if let Some(limit) = bucket.limit {
    //                 b = b.limit(limit);
    //             }

    //             b
    //         })
    //         .await;
    // }

    // framework = framework
    //     .group(&commands::groups::general::GENERAL_GROUP)
    //     .group(&commands::groups::config::CONFIG_GROUP)
    //     .group(&commands::groups::greenbook::GREENBOOK_GROUP)
    //     // .group(&commands::groups::rules::RULES_GROUP)
    //     .group(&commands::groups::roles::ROLES_GROUP)
    //     .group(&commands::groups::account::ACCOUNT_GROUP)
    //     .group(&commands::groups::moderation::MODERATION_GROUP)
    //     .group(&commands::groups::misc::MISC_GROUP)
    //     .group(&commands::groups::lastfm::LASTFM_GROUP);
    // debug!("Framework created");

    // let mut client = Client::builder(&config.discord_token)
    //     .cache_update_timeout(std::time::Duration::from_millis(500))
    //     .event_handler(handler::Handler)
    //     .application_id(config.application_id)
    //     .framework(framework)
    //     .intents(GatewayIntents::all())
    //     .await
    //     .expect("Err creating client");

    // let rr_state = Arc::new(Mutex::new(self::reaction_roles::State::load_set()));
    // let rules_state = Arc::new(Mutex::new(self::rules::State::load()));

    // let pool = PgPoolOptions::new()
    //     .max_connections(config.db_pool_max_size)
    //     .connect(&config.db_url)
    //     .await
    //     .expect("Could not setup db pool");

    // let opt_out = Arc::new(Mutex::new(OptOutStore::load_or_init()));

    // startup::init_xkcd(&config).await;

    // MESSAGE_REGEX
    //     .set(
    //         regex::Regex::new(
    //             r#"https://(?:discord.com|discordapp.com)/channels/(\d+)/(\d+)/(\d+)"#,
    //         )
    //         .expect("Could not compile quote link regex"),
    //     )
    //     .unwrap();

    // {
    //     let mut data = client.data.write().await;

    //     data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    //     data.insert::<ReactionRolesState>(rr_state);
    //     data.insert::<RulesState>(rules_state);
    //     data.insert::<OptOut>(Arc::clone(&opt_out));
    //     data.insert::<DatabasePool>(pool);
    //     data.insert::<ReqwestClient>(reqwest::Client::new());
    //     data.insert::<RunningState>(BotState {
    //         running_since: std::time::Instant::now(),
    //     });
    //     data.insert::<XkcdState>(XkcdIndexStorage::load_or_init());
    //     data.insert::<Config>(config);
    // }

    // startup::init(&client).await;

    // if let Err(why) = client.start().await {
    //     error!("Client error: {:?}", why);
    // }
}
