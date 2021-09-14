pub mod config;
pub mod error;

use std::sync::Arc;

use config::Config;

use error::TrashyStartupError;
use futures::stream::StreamExt;
use rand::{rngs::StdRng, SeedableRng};
use tokio::sync::Mutex;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{cluster::ShardScheme, Cluster, Event, EventTypeFlags, Intents};
use twilight_http::Client;
use twilight_model::application::{callback::InteractionResponse, interaction::Interaction};

#[derive(Clone)]
pub struct TrashyContext {
    pub rng: Arc<Mutex<StdRng>>,
    pub http: Arc<Client>,
}

pub struct TrashyBot;

impl TrashyBot {
    pub async fn run(config: Config) -> Result<(), TrashyStartupError> {
        let token = config.discord_token;
        let scheme = ShardScheme::Auto;

        let http = Client::new(token.clone());
        let current_user = http.current_user().exec().await?.model().await?;
        http.set_application_id(current_user.id.0.into());
        http.set_global_commands(&commands::commands())?
            .exec()
            .await?;

        let intents = Intents::GUILD_MESSAGES | Intents::DIRECT_MESSAGES;

        let flags = EventTypeFlags::MESSAGE_CREATE
            | EventTypeFlags::READY
            | EventTypeFlags::INTERACTION_CREATE;

        let (cluster, mut events) = Cluster::builder(&token, intents)
            .event_types(flags)
            .shard_scheme(scheme)
            .http_client(http.clone())
            .build()
            .await?;

        let cluster_spawn = cluster.clone();

        tokio::spawn(async move {
            cluster_spawn.up().await;
        });

        let cache = InMemoryCache::builder()
            .resource_types(ResourceType::MESSAGE)
            .build();

        let context = TrashyContext {
            rng: Arc::new(Mutex::new(StdRng::seed_from_u64(41237102))),
            http: Arc::new(http),
        };

        while let Some((_, event)) = events.next().await {
            cache.update(&event);
            tokio::spawn(handle_event(event, context.clone()));
        }

        Ok(())
    }
}

pub async fn handle_event(event: Event, ctx: TrashyContext) {
    match event {
        Event::Ready(ready) => {
            tracing::info!("In {} guilds!", ready.guilds.len());
        }
        Event::InteractionCreate(interaction) => {
            tracing::debug!("Interaction");
            match handle_slash(interaction.0, ctx.clone()).await {
                Ok(_) => tracing::debug!("interaction completed"),
                Err(e) => tracing::error!(?e, "interaction could not be completed"),
            }
        }
        _ => tracing::warn!(?event, "unsupported event!"),
    }
}

pub async fn handle_slash(
    interaction: Interaction,
    ctx: TrashyContext,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match interaction {
        Interaction::Ping(ping) => {
            ctx.http
                .interaction_callback(ping.id, &ping.token, &InteractionResponse::Pong)
                .exec()
                .await?;
            Ok(())
        }
        Interaction::ApplicationCommand(cmd) => {
            tracing::debug!("application command");
            let name = cmd.data.name.as_str();
            match name {
                "roll" => commands::roll::roll(cmd, &ctx).await,
                "choose" => commands::choose::choose(cmd, &ctx).await,
                unknown => tracing::warn!(?unknown, "unknown command"),
            }

            Ok(())
        }
        Interaction::MessageComponent(_cmd) => {
            tracing::debug!("message component not supported");

            Ok(())
        }
        _ => Err("unknown interaction type".into()),
    }
}

mod commands {
    use twilight_model::{
        application::command::{ChoiceCommandOptionData, Command, CommandOption},
        id::GuildId,
    };

    pub fn commands() -> Vec<Command> {
        vec![
            Command {
                id: None,
                application_id: None,
                guild_id: Some(GuildId(884438532322652251)),
                name: "roll".to_string(),
                default_permission: None,
                description: "Roll some die!".to_string(),
                options: vec![CommandOption::String(ChoiceCommandOptionData {
                    choices: vec![],
                    description: "specify which die you want to roll".to_string(),
                    name: "roll".to_string(),
                    required: true,
                })],
            },
            Command {
                id: None,
                application_id: None,
                guild_id: Some(GuildId(884438532322652251)),
                name: "choose".to_string(),
                default_permission: None,
                description: "Choose something!".to_string(),
                options: vec![
                    CommandOption::String(ChoiceCommandOptionData {
                        choices: vec![],
                        description: "specify which die you want to roll".to_string(),
                        name: "options".to_string(),
                        required: true,
                    }),
                    CommandOption::Integer(ChoiceCommandOptionData {
                        choices: vec![],
                        description: "specify how many options should be picked (default 1)"
                            .to_string(),
                        name: "pick".to_string(),
                        required: false,
                    }),
                ],
            },
            Command {
                id: None,
                application_id: None,
                guild_id: Some(GuildId(884438532322652251)),
                name: "sponge".to_string(),
                default_permission: None,
                description: "sPonGiFy sOmE wOrdS!".to_string(),
                options: vec![CommandOption::String(ChoiceCommandOptionData {
                    choices: vec![],
                    description: "specify what you want to spongify".to_string(),
                    name: "text".to_string(),
                    required: true,
                })],
            },
        ]
    }

    pub mod choose;
    pub mod roll;
}
