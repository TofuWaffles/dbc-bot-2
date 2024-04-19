use futures::poll;
use std::{collections::HashMap, sync::Arc};

use api::{BrawlStarsApi, GameApi};

use database::{Database, PgDatabase};
use poise::serenity_prelude::{self as serenity, futures::StreamExt};
use reminder::MatchReminders;
use tokio::sync::Mutex;
use tokio_util::time::DelayQueue;
use tournament_model::{SingleElimTournament, TournamentModel};

use commands::{
    manager_commands::ManagerCommands, marshal_commands::MarshalCommands,
    owner_commands::OwnerCommands, user_commands::UserCommands, CommandsContainer,
};

/// Utilities for interacting with the game API.
mod api;
/// All the commands that the bot can run.
///
/// Additionally, it contains the `CommandsContainer` trait that groups all the commands together
/// as well as checks used by various commands.
mod commands;
/// Traits and types used for interacting with the database.
mod database;
mod reminder;
/// Contains the tournament model, which is used to manage tournaments.
mod tournament_model;

/// Stores data used by the bot.
///
/// Accessible by all bot commands through Context.
pub struct Data<DB: Database, TM: TournamentModel, P: GameApi> {
    database: DB,
    tournament_model: TM,
    game_api: P,
    match_reminders: Arc<Mutex<MatchReminders>>,
}

/// Convenience type for the bot's data with generics filled in.
pub type BotData = Data<PgDatabase, SingleElimTournament, BrawlStarsApi>;

/// A thread-safe Error type used by the bot.
pub type BotError = Box<dyn std::error::Error + Send + Sync>;

/// A context that gives the bot information about the action that invoked it.
///
/// It also includes other useful data that the bot uses such as the database.
/// You can access the data in commands by using ``ctx.data()``.
pub type Context<'a> = poise::Context<'a, BotData, BotError>;

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
        MarshalCommands::get_commands_list(),
        UserCommands::get_commands_list(),
    ]
    .into_iter()
    .flatten()
    .collect();

    let match_reminders = Arc::new(Mutex::new(MatchReminders::new(
        DelayQueue::new(),
        HashMap::new(),
    )));
    let bot_data_match_reminders = match_reminders.clone();

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
                    match_reminders: bot_data_match_reminders,
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(discord_token, intents)
        .framework(framework)
        .await?;

    let http = client.http.clone();

    // Todo: revisit this later once the reminder feature has been laid out
    // Note that all errors in this block should be handled and reported properly (i.e. no unwraps)
    // so that the loop can continue, otherwise the task will die and no reminders will be sent
    let _ = tokio::spawn(async move {
        loop {
            let mut locked_match_reminders = match_reminders.lock().await;
            // Manual polling is needed because an await would otherwise hold up the Mutex until the next
            // reminder expires, which means no new reminders could be added in the meantime
            let expired_reminder_opt = match poll!(locked_match_reminders.reminder_times.next()) {
                std::task::Poll::Ready(expired_reminder_opt) => expired_reminder_opt,
                std::task::Poll::Pending => continue,
            };
            // The DelayQueue will return None if the queue is empty, in that case we just continue
            match expired_reminder_opt {
                Some(expired_reminder) => {
                    let match_id = expired_reminder.into_inner();
                    let channel_id = match &locked_match_reminders.matches.remove(&match_id) {
                        Some(reminder) => reminder.notification_channel_id.clone(),
                        None => todo!(),
                    };
                    let channel = match http
                        .clone()
                        .get_channel(channel_id.parse::<u64>().unwrap_or_default().into())
                        .await
                    {
                        Ok(channel) => channel,
                        Err(e) => todo!(),
                    };
                    let guild_channel = match channel.guild() {
                        Some(guild_channel) => guild_channel,
                        None => todo!(),
                    };
                    guild_channel
                        .say(
                            http.clone(),
                            format!(
                                "Match reminder {}",
                                locked_match_reminders
                                    .matches
                                    .get(&match_id)
                                    .unwrap()
                                    .discord_id_1
                            ),
                        )
                        .await
                        .unwrap();
                }
                None => continue,
            };
        }
    });

    client.start().await?;

    Ok(())
}
