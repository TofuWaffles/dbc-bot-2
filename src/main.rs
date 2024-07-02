use std::fs::File;
use tracing::{error, info, info_span, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

use api::{BrawlStarsApi, GameApi};

use database::{Database, PgDatabase};
use poise::{serenity_prelude as serenity, CreateReply};

use commands::{
    manager_commands::ManagerCommands, marshal_commands::MarshalCommands,
    owner_commands::OwnerCommands, user_commands::UserCommands, CommandsContainer,
};

use crate::log::discord_log_error;

/// Utilities for interacting with the game API.
mod api;
/// All the commands that the bot can run.
///
/// Additionally, it contains the `CommandsContainer` trait that groups all the commands together
/// as well as checks used by various commands.
mod commands;
/// Traits and types used for interacting with the database.
mod database;
mod log;

/// Stores data used by the bot.
///
/// Accessible by all bot commands through Context.
#[derive(Debug)]
pub struct Data<DB, P> {
    database: DB,
    game_api: P,
}

impl<DB, P> Data<DB, P>
where
    DB: Database,
    P: GameApi,
{
    fn new(database: DB, game_api: P) -> Self {
        Self { database, game_api }
    }
}

/// Convenience type for the bot's data with generics filled in.
pub type BotData = Data<PgDatabase, BrawlStarsApi>;

/// A thread-safe Error type used by the bot.
pub type BotError = Box<dyn std::error::Error + Send + Sync>;

/// A context that gives the bot information about the action that invoked it.
///
/// It also includes other useful data that the bot uses such as the database.
/// You can access the data in commands by using ``ctx.data()``.
pub type Context<'a> = poise::Context<'a, BotData, BotError>;

#[tokio::main]
async fn main() {
    if let Err(e) = setup_tracing() {
        panic!("Error trying to setup tracing: {}", e);
    }

    if let Err(e) = run().await {
        panic!("Error trying to run the bot: {}", e);
    }
}

/// The main function that runs the bot.
async fn run() -> Result<(), BotError> {
    let setup_span = info_span!("bot_setup");
    let _guard = setup_span.enter();
    // Load the .env file only in the development environment (bypassed with the --release flag)
    #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    let discord_token =
        std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN as an environment variable");
    info!("Successfully loaded Discord Token");
    let brawl_stars_token = std::env::var("BRAWL_STARS_TOKEN")
        .expect("Expected BRAWL_STARS_TOKEN as an environment variable");
    info!("Successfully loaded Brawl Stars Token");

    let pg_database = PgDatabase::connect().await.unwrap();
    info!("Successfully connected to the database");
    let brawl_stars_api = BrawlStarsApi::new(&brawl_stars_token);

    let commands = vec![
        OwnerCommands::get_all(),
        ManagerCommands::get_all(),
        MarshalCommands::get_all(),
        UserCommands::get_all(),
    ]
    .into_iter()
    .flatten()
    .collect();

    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            on_error: |error| {
                Box::pin(async move {
                    // These error variants aren't really errors and can be safely ignored
                    match error {
                        poise::FrameworkError::NotAnOwner { .. } => return,
                        poise::FrameworkError::GuildOnly { .. } => return,
                        poise::FrameworkError::DmOnly { .. } => return,
                        poise::FrameworkError::NsfwOnly { .. } => return,
                        poise::FrameworkError::CommandCheckFailed { .. } => return,
                        poise::FrameworkError::UnknownCommand { .. } => return,
                        _ => (),
                    }
                    error!("Error in command: {:?}", error);
                    let ctx = match error.ctx() {
                        Some(ctx) => ctx,
                        None => {
                            error!("No context in this error");
                            return;
                        },
                    };
                    match ctx.send(CreateReply::default().content("Something went wrong. Please let the bot maintainers know if the issue persists.").ephemeral(true)).await {
                        Ok(_) => (),
                        Err(e) => error!("Error sending generic error message to user: {}", e)
                    }
                    let guild_id = match ctx.guild_id() {
                        Some(guild_id) => guild_id.to_string(),
                        None => {
                            warn!("No guild id in this error context. Cannot send error message to log channel.");
                            return;
                        },
                    };

                    let player_tournaments = match ctx.data().database.get_player_active_tournaments(&guild_id, &ctx.author().id.to_string()).await {
                        Ok(tournament) => tournament,
                        Err(e) => {
                            error!("Error getting player active tournament for user {}. Cannot send error message to log channel: {}", ctx.author().id, e);
                            return;
                        },
                    };

                    let user = match ctx.data().database.get_user_by_discord_id(&ctx.author().id.to_string()).await {
                        Ok(user) => user,
                        Err(e) => {
                            error!("Error getting user for user {}. Cannot send error message to log channel: {}", ctx.author().id, e);
                            return;
                        },
                    };

                    match discord_log_error(
                        ctx,
                        &error.to_string(),
                        user,
                        player_tournaments,
                        ).await {
                        Ok(_) => (),
                        Err(e) => error!("Error sending error message to log channel: {:?}", e),
                    };
                })
            },
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                println!("Ready as {}", ready.user.name);
                Ok(Data::new(pg_database, brawl_stars_api))
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(discord_token, intents)
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}

/// Sets up the tracing subscriber for the bot.
fn setup_tracing() -> Result<(), BotError> {
    if cfg!(debug_assertions) {
        let filter = EnvFilter::from_default_env()
            .add_directive("none".parse()?)
            .add_directive("dbc_bot_2=info".parse()?);

        tracing_subscriber::fmt::fmt()
            .with_env_filter(filter)
            .with_span_events(FmtSpan::NONE)
            .pretty()
            .init();

        return Ok(());
    }

    let log_file = File::create("debug.log")?;

    // Set up tracing with a filter that only logs errors in production
    tracing_subscriber::fmt::fmt()
        .with_span_events(FmtSpan::NONE)
        .with_max_level(LevelFilter::ERROR)
        .with_writer(log_file)
        .pretty()
        .init();

    Ok(())
}
