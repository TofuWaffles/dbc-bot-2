use std::marker::PhantomData;

use poise::serenity_prelude as serenity;

mod error;
mod tournament_model;
mod database;
mod commands;

use database::{Database, PgDatabase};
use tournament_model::{SingleElimTournament, TournamentModel};

use commands::{manager_commands::ManagerCommands, CommandsContainer};

/// Stores data used by the bot.
///
/// Accessible by all bot commands through Context.
pub struct BotData<DB: Database, TM: TournamentModel<DB>> {
    tournament_model: TM,
    phantom: PhantomData<DB>
}

/// A thread-safe Error type used by the bot.
pub type BotError = Box<dyn std::error::Error + Send + Sync>;

/// A context that gives the bot information about the action that invoked it.
///
/// It also includes other useful data that the bot uses such as the database.
/// You can access the data in commands by using ``ctx.data()``.
pub type Context<'a> = poise::Context<'a, BotData<PgDatabase, SingleElimTournament<PgDatabase>>, BotError>;

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

    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let intents = serenity::GatewayIntents::non_privileged();

    let pg_database = PgDatabase::new().await;
    let dbc_tournament = SingleElimTournament { database: pg_database };

    let manager_commands = ManagerCommands::get_commands_list(); 

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: manager_commands,
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(BotData {
                    tournament_model: dbc_tournament,
                    phantom: PhantomData,
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
