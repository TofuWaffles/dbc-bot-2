use api::{BrawlStarsApi, GameApi};

use database::{Database, PgDatabase};
use poise::serenity_prelude as serenity;
use tournament_model::{SingleElimTournament, TournamentModel};

use commands::{
    manager_commands::ManagerCommands, owner_commands::OwnerCommands, CommandsContainer,
};

/// Contains the types used to interact with the game API.
mod api;
/// Contains all the commands that the bot can run.
///
/// Additionally, it contains the `CommandsContainer` trait that groups all the commands together
/// as well as checks used by various commands.
mod commands;
/// Contains traits and types for database implementation.
mod database;
/// Contains models used by both the tournament model and the database.
mod models;
/// Contains the tournament model, which is used to manage tournaments.
mod tournament_model;

/// Stores data used by the bot.
///
/// Accessible by all bot commands through Context.
pub(crate) struct Data<DB: Database, TM: TournamentModel, P: GameApi> {
    database: DB,
    tournament_model: TM,
    game_api: P,
}

/// Convenience type for the bot's data with generics filled in.
type BotData = Data<PgDatabase, SingleElimTournament, BrawlStarsApi>;

/// A thread-safe Error type used by the bot.
pub(crate) type BotError = Box<dyn std::error::Error + Send + Sync>;

/// A context that gives the bot information about the action that invoked it.
///
/// It also includes other useful data that the bot uses such as the database.
/// You can access the data in commands by using ``ctx.data()``.
pub(crate) type Context<'a> = poise::Context<'a, BotData, BotError>;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        panic!("Error trying to run the bot: {}", e);
    }
}

/// The main function that runs the bot.
async fn run() -> Result<(), BotError> {
    // Load the .env file only in the development environment (bypassed with the --release flag)
    #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    let discord_token =
        std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN as an environment variable");
    let brawl_stars_token = std::env::var("BRAWL_STARS_TOKEN")
        .expect("Expected BRAWL_STARS_TOKEN as an environment variable");

    let pg_database = PgDatabase::connect().await.unwrap();
    let dbc_tournament = SingleElimTournament {};
    let brawl_stars_api = BrawlStarsApi::new(&brawl_stars_token);

    let commands = vec![
        OwnerCommands::get_commands_list(),
        ManagerCommands::get_commands_list(),
    ]
    .into_iter()
    .flatten()
    .collect();

    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    tournament_model: dbc_tournament,
                    database: pg_database,
                    game_api: brawl_stars_api,
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(discord_token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();

    Ok(())
}
