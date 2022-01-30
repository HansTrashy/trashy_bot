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

pub mod config;
pub mod error;
pub mod util;

use std::sync::Arc;

use config::Config;

use error::TrashyStartupError;
use futures::stream::StreamExt;
use rand::{rngs::StdRng, SeedableRng};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::sync::Mutex;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{cluster::ShardScheme, Cluster, Event, EventTypeFlags, Intents};
use twilight_http::Client;
use twilight_model::application::{callback::InteractionResponse, interaction::Interaction};

use crate::error::TrashyCommandError;

#[derive(Clone)]
/// Context struct for the trashy bot
///
/// the context contains ways to access all necessary datastores/interaction avenues
pub struct TrashyContext {
    /// a source for rng
    pub rng: Arc<Mutex<StdRng>>,
    /// communication with the discord api
    pub http: Arc<Client>,
    /// the database pool
    pub db: PgPool,
}

/// The TrashyBot itself
pub struct TrashyBot;

impl TrashyBot {
    /// this function does necessary setup and runs the bot
    ///
    /// # Errors
    ///
    /// this function errors if any startup conditions are not met, see `TrashyStartupError`
    pub async fn run(config: Config) -> Result<(), TrashyStartupError> {
        let token = config.discord_token;
        let scheme = ShardScheme::Auto;

        let http = Arc::new(Client::new(token.clone()));
        // let current_user = http.current_user().exec().await?.model().await?;
        // http.set_application_id(current_user.id.0.into());
        // http.set_global_commands(&commands::commands())?
        //     .exec()
        //     .await?;

        let intents = Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES;

        let flags = EventTypeFlags::MESSAGE_CREATE
            | EventTypeFlags::READY
            | EventTypeFlags::INTERACTION_CREATE;

        let (cluster, mut events) = Cluster::builder(token, intents)
            .event_types(flags)
            .shard_scheme(scheme)
            .http_client(Arc::clone(&http))
            .build()
            .await?;

        let cluster = Arc::new(cluster);

        let cluster_spawn = Arc::clone(&cluster);

        tokio::spawn(async move {
            cluster_spawn.up().await;
        });

        let cache = InMemoryCache::builder()
            .resource_types(ResourceType::MESSAGE)
            .build();

        let pool = PgPoolOptions::new()
            .max_connections(config.db_pool_max_size)
            .connect(&config.db_url)
            .await?;

        sqlx::migrate!().run(&pool).await?;

        let context = TrashyContext {
            rng: Arc::new(Mutex::new(StdRng::seed_from_u64(41237102))),
            http: Arc::clone(&http),
            db: pool,
        };

        while let Some((_, event)) = events.next().await {
            cache.update(&event);
            tokio::spawn(handle_event(event, context.clone()));
        }

        Ok(())
    }
}

/// the event handler function
///
/// this function listens to all events and dispatches them to their corresponding handler
pub async fn handle_event(event: Event, ctx: TrashyContext) {
    match event {
        Event::Ready(ready) => {
            tracing::info!("In {} guilds!", ready.guilds.len());
        }
        Event::InteractionCreate(interaction) => {
            tracing::debug!("Interaction");
            // match handle_interaction(interaction.0, ctx.clone()).await {
            //     Ok(_) => tracing::debug!("interaction completed"),
            //     Err(e) => tracing::error!(?e, "interaction could not be completed"),
            // }
        }
        _ => tracing::warn!(?event, "unsupported event!"),
    }
}

// /// the interaction handler
// ///
// /// this function handles dispatching of different interaction types
// pub async fn handle_interaction(
//     interaction: Interaction,
//     ctx: TrashyContext,
// ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     match interaction {
//         Interaction::Ping(ping) => {
//             ctx.http
//                 .interaction_callback(ping.id, &ping.token, &InteractionResponse::Pong)
//                 .exec()
//                 .await?;
//             Ok(())
//         }
//         Interaction::ApplicationCommand(cmd) => {
//             tracing::debug!("application command");
//             let name = cmd.data.name.as_str();
//             let result = match name {
//                 "roll" => commands::roll::roll(cmd, &ctx).await,
//                 "choose" => commands::choose::choose(cmd, &ctx).await,
//                 "sponge" => commands::spongebob::sponge(cmd, &ctx).await,
//                 "remindme" => commands::remindme::remindme(cmd, &ctx).await,
//                 unknown => {
//                     tracing::warn!(?unknown, "unknown command");
//                     Err(TrashyCommandError::UnknownCommand(unknown.to_string()))
//                 }
//             };

//             Ok(result?)
//         }
//         Interaction::MessageComponent(_cmd) => {
//             tracing::debug!("message component not supported");

//             Ok(())
//         }
//         _ => Err("unknown interaction type".into()),
//     }
// }

// mod commands {
//     use twilight_model::{
//         application::command::{ChoiceCommandOptionData, Command, CommandOption, CommandType},
//         id::{CommandVersionId, GuildId},
//     };
//     use twilight_util::builder::command::{CommandBuilder, IntegerBuilder, StringBuilder};

//     pub fn commands() -> Vec<Command> {
//         [
//             CommandBuilder::new(
//                 "roll".to_string(),
//                 "Roll some die!".to_string(),
//                 CommandType::ChatInput,
//             )
//             .guild_id(GuildId::new(884438532322652251).unwrap())
//             .option(
//                 StringBuilder::new(
//                     "roll".to_string(),
//                     "specify which die you want to roll".to_string(),
//                 )
//                 .required(true)
//                 .build(),
//             )
//             .build(),
//             CommandBuilder::new(
//                 "choose".to_string(),
//                 "Choose something!".to_string(),
//                 CommandType::ChatInput,
//             )
//             .guild_id(GuildId::new(884438532322652251).unwrap())
//             .option(
//                 StringBuilder::new(
//                     "options".to_string(),
//                     "specify which die you want to roll".to_string(),
//                 )
//                 .required(true)
//                 .build(),
//             )
//             .option(
//                 IntegerBuilder::new(
//                     "pick".to_string(),
//                     "specify how many options should be picked (default 1)".to_string(),
//                 )
//                 .build(),
//             )
//             .build(),
//             CommandBuilder::new(
//                 "sponge".to_string(),
//                 "sPonGiFy sOmE wOrdS!".to_string(),
//                 CommandType::ChatInput,
//             )
//             .guild_id(GuildId::new(884438532322652251).unwrap())
//             .option(
//                 StringBuilder::new(
//                     "text".to_string(),
//                     "specify what you want to spongify".to_string(),
//                 )
//                 .required(true)
//                 .build(),
//             )
//             .build(),
//             CommandBuilder::new(
//                 "remindme".to_string(),
//                 "let the bot remind you!".to_string(),
//                 CommandType::ChatInput,
//             )
//             .guild_id(GuildId::new(884438532322652251).unwrap())
//             .option(
//                 StringBuilder::new("when".to_string(), "date or duration".to_string())
//                     .required(true)
//                     .build(),
//             )
//             .option(
//                 StringBuilder::new(
//                     "message".to_string(),
//                     "what should i remind you about?".to_string(),
//                 )
//                 .build(),
//             )
//             .build(),
//         ]
//         .into()
//     }

//     pub mod choose;
//     pub mod roll;
//     pub mod spongebob;
//     //TODO: quote (implement as soon as MESSAGE commands are supported by twilight)
//     //TODO: favs (implement as soon as MESSAGE commands are supported by twilight)

//     //TODO: remindme
//     pub mod remindme;

//     //TODO: xkcds

//     //TODO: poll

//     //TODO: fighting

//     //TODO: userinfo

//     //TODO: copypasta system?

//     //TODO: lastfm?
// }

// /// Database models
// pub mod models {
//     /// reminder database model
//     pub mod reminder;
// }
