use chrono::DateTime;
use poise::{serenity_prelude::CreateEmbed, CreateReply};
use prettytable::{row, Table};
use tracing::{instrument, warn};

use crate::{
    database::{
        models::{Match, PlayerNumber, TournamentStatus},
        Database,
    },
    BotData, BotError, Context,
};

use super::{checks::is_marshal_or_higher, CommandsContainer};

/// CommandsContainer for the Marshal commands
pub struct MarshalCommands;

impl CommandsContainer for MarshalCommands {
    type Data = BotData;
    type Error = BotError;

    fn get_all() -> Vec<poise::Command<Self::Data, Self::Error>> {
        vec![get_tournament(), list_active_tournaments(), next_round()]
    }
}

/// Get information about a tournament.
#[poise::command(
    slash_command,
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

/// List all currently active tournaments.
#[poise::command(
    slash_command,
    guild_only,
    check = "is_marshal_or_higher",
    rename = "tournaments"
)]
#[instrument]
async fn next_round(ctx: Context<'_>, tournament_id: i32) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap().to_string();

    let tournament = match ctx
        .data()
        .database
        .get_tournament(&guild_id, &tournament_id)
        .await?
    {
        Some(tournament) => tournament,
        None => {
            ctx.send(CreateReply::default().content("No tournament found with the given ID. Try again with an existing tournament ID.").ephemeral(true)).await?;
            return Ok(());
        }
    };

    if tournament.status != TournamentStatus::Started {
        ctx.send(CreateReply::default().content("This tournament is not currently active. Please try again when the tournament is active again.").ephemeral(true)).await?;
        return Ok(());
    }

    if tournament.current_round == tournament.rounds {
        ctx.send(CreateReply::default().content("Unable to advance to the next round. This tournament is currently on its final round.").ephemeral(true)).await?;
    }

    let brackets = ctx
        .data()
        .database
        .get_matches_by_tournament(&tournament_id, Some(&tournament.current_round))
        .await?;

    let (with_winners, without_winners): (Vec<Match>, Vec<Match>) = brackets
        .into_iter()
        .partition(|bracket| bracket.winner.is_some());

    if without_winners.len() > 0 {
        // TODO: Show unfinished matches as a table or a CSV file
        ctx.send(CreateReply::default().content("Unable to advance to the next round. Some players have not finished their matches yet!").ephemeral(true)).await?;
        return Ok(());
    }

    let round = tournament.current_round + 1;
    let next_round_brackets = generate_next_round(with_winners, round)?;

    for bracket in next_round_brackets {
        ctx.data()
            .database
            .create_match(
                &tournament_id,
                &round,
                &bracket.sequence_in_round,
                bracket.player_1_type,
                bracket.player_2_type,
                Some(&bracket.discord_id_1.unwrap()),
                Some(&bracket.discord_id_2.unwrap()),
            )
            .await?;
    }

    ctx.data().database.next_round(&round).await?;

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Successfully advanced the tournament with ID {} to the round {}.",
                tournament_id, round
            ))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

fn generate_next_round(brackets: Vec<Match>, round: i32) -> Result<Vec<Match>, BotError> {
    let mut next_round_brackets = Vec::with_capacity(brackets.len() / 2);
    let tournament_id = brackets[0].tournament_id.to_owned();
    let mut brackets_iter = brackets.into_iter();

    for _i in 1..=next_round_brackets.len() {
        let old_bracket_1 = brackets_iter.next().unwrap();
        let old_bracket_2 = brackets_iter.next().unwrap();

        let discord_id_1 = match old_bracket_1.winner.unwrap() {
            PlayerNumber::Player1 => old_bracket_1.discord_id_1,
            PlayerNumber::Player2 => old_bracket_1.discord_id_2,
        };
        let discord_id_2 = match old_bracket_2.winner.unwrap() {
            PlayerNumber::Player1 => old_bracket_2.discord_id_1,
            PlayerNumber::Player2 => old_bracket_2.discord_id_2,
        };
        let new_sequence = (old_bracket_1.sequence_in_round as f32 / 2.0).ceil() as i32;

        if new_sequence != (old_bracket_2.sequence_in_round / 2) {
            return Err(format!("Error generating matches for the next round. Previous round matches do not match:\n\nMatch ID 1: {}\nMatch ID 2: {}", old_bracket_1.match_id, old_bracket_2.match_id).into());
        }

        let match_id = Match::generate_id(&tournament_id, &round, &new_sequence);

        next_round_brackets.push(Match::new(
            match_id,
            tournament_id,
            round,
            new_sequence,
            discord_id_1,
            discord_id_2,
        ))
    }

    Ok(next_round_brackets)
}
