use poise::{serenity_prelude as serenity, Command};

use database::{Database, PgDatabase};
use tournament_model::{SingleElimTournament, TournamentModel};

use commands::{
    manager_commands::ManagerCommands, owner_commands::OwnerCommands, CommandsContainer,
};

/// Contains all the commands that the bot can run.
///
/// Additionally, it contains the `CommandsContainer` trait that groups all the commands together
/// as well as checks used by various commands.
mod commands;
/// Contains traits and types for database implementation.
mod database;
/// Contains the tournament model, which is used to manage tournaments.
mod tournament_model;
/// Contains models used by both the tournament model and the database.
mod models;
/// Contains the types used to interact with the game API.
mod api;


/// Stores data used by the bot.
///
/// Accessible by all bot commands through Context.
pub struct BotData<DB: Database, TM: TournamentModel> {
    database: DB,
    tournament_model: TM,
}

/// A thread-safe Error type used by the bot.
pub type BotError = Box<dyn std::error::Error + Send + Sync>;

/// A context that gives the bot information about the action that invoked it.
///
/// It also includes other useful data that the bot uses such as the database.
/// You can access the data in commands by using ``ctx.data()``.
pub type Context<'a> = poise::Context<'a, BotData<PgDatabase, SingleElimTournament>, BotError>;

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

    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN as an environment variable");

    let intents = serenity::GatewayIntents::non_privileged();

    let pg_database = PgDatabase::connect().await;
    let dbc_tournament = SingleElimTournament {};

    let commands = vec![
        OwnerCommands::get_commands_list(),
        ManagerCommands::get_commands_list(),
    ]
    .into_iter()
    .flatten()
    .collect();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(BotData {
                    tournament_model: dbc_tournament,
                    database: pg_database,
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();

    Ok(())
}
