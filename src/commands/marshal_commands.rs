use chrono::DateTime;
use poise::{serenity_prelude::CreateEmbed, CreateReply};
use prettytable::{row, Table};
use tracing::{instrument, warn};

use crate::{database::Database, BotData, BotError, Context};

use super::{checks::is_marshal_or_higher, CommandsContainer};

/// CommandsContainer for the Marshal commands
pub struct MarshalCommands;

impl CommandsContainer for MarshalCommands {
    type Data = BotData;
    type Error = BotError;

    fn get_commands_list() -> Vec<poise::Command<Self::Data, Self::Error>> {
        vec![get_tournament(), list_active_tournaments()]
    }
}

/// Get information about a tournament.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    check = "is_marshal_or_higher",
    rename = "tournament"
)]
#[instrument]
async fn get_tournament(ctx: Context<'_>, tournament_id: i32) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap().to_string();

    let tournament = ctx
        .data()
        .database
        .get_tournament(&guild_id, &tournament_id)
        .await?;

    match tournament {
        Some(tournament) => {
            let start_time_str = match tournament.start_time {
                Some(start_time) => DateTime::from_timestamp(start_time, 0)
                    .unwrap_or_default()
                    .to_rfc2822(),
                None => "Not started".to_string(),
            };
            ctx.send(
                CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .title(tournament.name)
                            .field("ID", tournament.tournament_id.to_string(), true)
                            .field(
                                "Created At",
                                DateTime::from_timestamp(tournament.created_at, 0)
                                    .unwrap_or_default()
                                    .to_rfc2822(),
                                true,
                            )
                            .field("Started At", start_time_str, true),
                    )
                    .ephemeral(true),
            )
            .await?;
        }
        None => {
            ctx.send(
                CreateReply::default()
                    .content("A tournament with that id was not found")
                    .ephemeral(true),
            )
            .await?;
            warn!("Tournament with id {} not found", tournament_id);
        }
    };

    Ok(())
}

/// List all currently active tournaments.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    check = "is_marshal_or_higher",
    rename = "tournaments"
)]
#[instrument]
async fn list_active_tournaments(ctx: Context<'_>) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap().to_string();

    let tournaments = ctx
        .data()
        .database
        .get_active_tournaments(&guild_id)
        .await?;

    if tournaments.is_empty() {
        ctx.send(
            CreateReply::default()
                .content("There are no currently active tournaments. You can create one by using the /create_tournament command")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let mut table = Table::new();
    table.set_titles(row!["ID", "Name", "Created At", "Status"]);

    tournaments.iter().for_each(|tournament| {
        table.add_row(row![
            tournament.tournament_id,
            tournament.name,
            DateTime::from_timestamp(tournament.created_at, 0)
                .unwrap_or_default()
                .to_rfc2822(),
            tournament.status,
        ]);
    });

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Here are the currently active tournaments\n```\n{}\n```",
                table.to_string()
            ))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
