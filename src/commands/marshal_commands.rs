use chrono::DateTime;
use poise::{
    serenity_prelude::{CreateEmbed, User},
    CreateReply,
};
use prettytable::{row, Table};
use tracing::{instrument, warn};

use crate::{
    database::{
        models::{Match, PlayerNumber, TournamentStatus},
        Database,
    },
    log::discord_log_info,
    BotData, BotError, Context,
};

use super::{checks::is_marshal_or_higher, CommandsContainer};

/// CommandsContainer for the Marshal commands
pub struct MarshalCommands;

impl CommandsContainer for MarshalCommands {
    type Data = BotData;
    type Error = BotError;

    fn get_all() -> Vec<poise::Command<Self::Data, Self::Error>> {
        vec![
            get_tournament(),
            list_active_tournaments(),
            next_round(),
            set_map(),
            disqualify(),
        ]
    }
}

/// Get information about a tournament.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
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
    rename = "list_tournaments"
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

/// Set the map for a given tournament.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
#[instrument]
async fn set_map(ctx: Context<'_>, tournament_id: i32, map: String) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let tournament = match ctx
        .data()
        .database
        .get_tournament(&guild_id, &tournament_id)
        .await?
    {
        Some(tournament) => tournament,
        None => {
            ctx.send(
                CreateReply::default()
                    .content(format!(
                    "A tournament with the ID {} was not found. Please try again with another ID",
                    tournament_id
                ))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    ctx.data().database.set_map(&tournament_id, &map).await?;

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Sucessfully set the map of tournament {} to {}",
                tournament_id, map
            ))
            .ephemeral(true),
    )
    .await?;

    discord_log_info(
        ctx,
        &format!("A new map has been set for tournament {}", tournament.name),
        vec![
            (
                "Tournament ID",
                &tournament.tournament_id.to_string(),
                false,
            ),
            ("Tournament name", &tournament.name, false),
            ("Map", &map, false),
            ("Set by", &ctx.author().name, false),
        ],
    )
    .await?;

    Ok(())
}

/// Get the information about a match from a match ID or user.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
async fn get_match(
    ctx: Context<'_>,
    match_id: Option<String>,
    player: Option<User>,
) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let bracket;

    match match_id {
        Some(match_id) => {
            bracket = ctx.data().database.get_match_by_id(&match_id).await?;
        }
        None => match player {
            Some(player) => {
                let player_active_tournaments = ctx
                    .data()
                    .database
                    .get_player_active_tournaments(&guild_id, &player.id.to_string())
                    .await?;
                if player_active_tournaments.len() < 1 {
                    ctx.send(
                        CreateReply::default()
                            .content(format!(
                                "The player <@{}> is not currently in any active tournaments.",
                                player.id.to_string()
                            ))
                            .ephemeral(true),
                    )
                    .await?;

                    return Ok(());
                }
                bracket = ctx
                    .data()
                    .database
                    .get_match_by_player(
                        &player_active_tournaments[0].tournament_id,
                        &player.id.to_string(),
                    )
                    .await?;
            }
            None => {
                ctx.send(CreateReply::default().content("You must provide either a match ID or a player ID to run this command.").ephemeral(true)).await?;
                return Ok(());
            }
        },
    };

    match bracket {
        Some(bracket) => {
            ctx.send(
                CreateReply::default()
                    .content("")
                    .embed(
                        CreateEmbed::new()
                            .title(format!("Match {}", bracket.match_id))
                            .fields(vec![
                                ("Tournament ID", bracket.tournament_id.to_string(), false),
                                ("Round", bracket.round.to_string(), false),
                                (
                                    "Player 1",
                                    match bracket.discord_id_1 {
                                        Some(player_id) => format!("<@{}>", player_id),
                                        None => "No player".to_string(),
                                    },
                                    false,
                                ),
                                (
                                    "Player 2",
                                    match bracket.discord_id_2 {
                                        Some(player_id) => format!("<@{}>", player_id),
                                        None => "No player".to_string(),
                                    },
                                    false,
                                ),
                                ("Winner", format!("<@{:#?}>", bracket.winner), false),
                            ]),
                    )
                    .ephemeral(true),
            )
            .await?
        }
        None => {
            ctx.send(
                CreateReply::default()
                    .content("No match found for the given ID or player.")
                    .ephemeral(true),
            )
            .await?
        }
    };

    Ok(())
}

/// Disqualify a player from a given tournament.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
async fn disqualify(ctx: Context<'_>, tournament_id: i32, player: User) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let tournament = match ctx
        .data()
        .database
        .get_tournament(&guild_id, &tournament_id)
        .await?
    {
        Some(tournament) => tournament,
        None => {
            ctx.send(CreateReply::default().content(format!("A tournament with the ID {} was not found. Try again with an existing tournament ID.", tournament_id)).ephemeral(true)).await?;
            return Ok(());
        }
    };

    let bracket = match ctx
        .data()
        .database
        .get_match_by_player(&tournament.tournament_id, &player.id.to_string())
        .await?
    {
        Some(bracket) => bracket,
        None => {
            ctx.send(
                CreateReply::default()
                    .content(format!(
                        "An unfinished match could not be found for <@{}> in tournament {}.",
                        player.id.to_string(),
                        tournament_id
                    ))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    if player.id.to_string() == bracket.discord_id_1.unwrap() {
        ctx.data()
            .database
            .set_winner(&bracket.match_id, PlayerNumber::Player2)
            .await?;
    } else {
        ctx.data()
            .database
            .set_winner(&bracket.match_id, PlayerNumber::Player1)
            .await?;
    }

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Successfully disqualified <@{}> from match {}",
                player.id.to_string(),
                bracket.match_id
            ))
            .ephemeral(true),
    )
    .await?;

    discord_log_info(
        ctx,
        &format!(
            "Player <@{}> was disqualified from tournament {}",
            player.id.to_string(),
            tournament.tournament_id
        ),
        vec![
            ("Disqualified player", &format!("<@{}>", player.id.to_string()), false),
            ("Match ID", &bracket.match_id, false),
            ("Tournament ID", &tournament.tournament_id.to_string(), false),
            ("Tournament name", &tournament.name, false),
            ("Disqualified by", &ctx.author().name, false),
        ],
    )
    .await?;

    Ok(())
}

/// List all currently active tournaments.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
#[instrument]
async fn next_round(
    ctx: Context<'_>,
    tournament_id: i32,
    map: Option<String>,
) -> Result<(), BotError> {
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
    let new_brackets_count = next_round_brackets.len();

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

    ctx.data().database.next_round(&tournament_id).await?;

    if let Some(map) = map {
        ctx.data().database.set_map(&tournament_id, &map).await?;
    }

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Successfully advanced the tournament with ID {} to the round {}.",
                tournament_id, round
            ))
            .ephemeral(true),
    )
    .await?;

    discord_log_info(
        ctx,
        &format!(
            "Tournament {} has advanced to round {}.",
            tournament.name,
            round
        ),
        vec![
            ("Tournament ID", &tournament.tournament_id.to_string(), false),
            ("Tournament name", &tournament.name, false),
            ("New round", &round.to_string(), false),
            ("Number of matches", &new_brackets_count.to_string(), false),
            ("Advanced by", &ctx.author().name, false),
        ],
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
