use super::{checks::is_marshal_or_higher, CommandsContainer};
use crate::database::models::{Match, PlayerType, TournamentStatus};
use crate::database::{MatchDatabase, TournamentDatabase, UserDatabase};
use crate::{
    log::{self, Log},
    utils::shorthand::BotContextExt,
    BotContext, BotData, BotError,
};
use anyhow::anyhow;
use chrono::DateTime;
use poise::{
    serenity_prelude::{CreateEmbed, User},
    CreateReply,
};
use prettytable::{row, Table};
use tracing::{instrument, warn};

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
            pause_tournament(),
            unpause_tournament(),
            get_match(),
            set_map(),
            disqualify(),
        ]
    }
}

/// Get information about a tournament.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
#[instrument]
async fn get_tournament(ctx: BotContext<'_>, tournament_id: i32) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap().to_string();

    let tournament = ctx
        .data()
        .database
        .get_tournament(&guild_id, tournament_id)
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
                    .embed(CreateEmbed::new().title(tournament.name).fields(
                        vec![("ID", tournament.tournament_id.to_string(), true),
                        ("Status", tournament.status.to_string(), true),
                        ("Rounds", tournament.rounds.to_string(), true),
                        ("Current Round", tournament.current_round.to_string(), true),
                        ("Wins Required Per Round", tournament.wins_required.to_string(), true),
                        ("Map", format!("{:#?}", tournament.map), true),
                            (
                                "Created At",
                                DateTime::from_timestamp(tournament.created_at, 0)
                                    .unwrap_or_default()
                                    .to_rfc2822(),
                                true,
                            ),
                            ("Started At", start_time_str, true),],
                    ))
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
async fn list_active_tournaments(ctx: BotContext<'_>) -> Result<(), BotError> {
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
                table
            ))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Set the map for a given tournament.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
#[instrument]
async fn set_map(ctx: BotContext<'_>, tournament_id: i32) -> Result<(), BotError> {
    let msg = ctx
        .send(CreateReply::default().embed(CreateEmbed::default().description("Loading maps...")))
        .await?;
    let guild_id = ctx.guild_id().unwrap().to_string();
    let tournament = match ctx
        .data()
        .database
        .get_tournament(&guild_id, tournament_id)
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
    let mode = tournament.mode;
    let map = ctx.map_selection(&msg, &mode).await?;
    ctx.data()
        .database
        .set_map(tournament_id, &map.clone().into())
        .await?;

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Sucessfully set the map of tournament {} to {}",
                tournament_id, &map.name
            ))
            .ephemeral(true),
    )
    .await?;
    let description = format!(
        r#"
The map for the tournament has been set to **{}**.
Tournament ID: {}.
Tournament name: {}.
Map: {}
Set by {}."#,
        map.name,
        tournament_id,
        tournament.name,
        map.name,
        ctx.author().name
    );
    ctx.log(
        "Map set successfully!",
        description,
        log::State::SUCCESS,
        log::Model::TOURNAMENT,
    )
    .await?;
    Ok(())
}

/// Get the information about a match from a match ID or user.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
async fn get_match(
    ctx: BotContext<'_>,
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
                if player_active_tournaments.is_empty() {
                    ctx.send(
                        CreateReply::default()
                            .content(format!(
                                "The player <@{}> is not currently in any active tournaments.",
                                player.id
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
                        player_active_tournaments[0].tournament_id,
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
                                    format!("{:#?}", bracket.match_players.get(0)),
                                    false,
                                ),
                                (
                                    "Player 2",
                                    format!("{:#?}", bracket.match_players.get(1)),
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

/// Manually pause a tournament and prevent any progress on it.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
async fn pause_tournament(ctx: BotContext<'_>, tournament_id: i32) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap().to_string();

    let tournament = match ctx
        .data()
        .database
        .get_tournament(&guild_id, tournament_id)
        .await?
    {
        Some(tournament) => tournament,
        None => {
            ctx.send(CreateReply::default().content(format!("No tournament found for the given ID {}. Try again with a different tournament ID.", tournament_id)).ephemeral(true)).await?;
            return Ok(());
        }
    };

    if tournament.status == TournamentStatus::Paused {
        ctx.send(
            CreateReply::default()
                .content(format!(
                    "The tournament with ID {} is already paused.",
                    tournament_id
                ))
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    ctx.data()
        .database
        .set_tournament_status(tournament_id, TournamentStatus::Paused)
        .await?;

    ctx.send(CreateReply::default()
             .content(format!("Successfully paused tournament with ID {}.\n\nNo progress can be made on the tournament until is is unpaused with the /unpause_tournament command.", tournament_id))
             .ephemeral(true)
        ).await?;

    Ok(())
}

/// Unpause a tournament so that progress can be made on it again.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
async fn unpause_tournament(ctx: BotContext<'_>, tournament_id: i32) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap().to_string();

    let tournament = match ctx
        .data()
        .database
        .get_tournament(&guild_id, tournament_id)
        .await?
    {
        Some(tournament) => tournament,
        None => {
            ctx.send(CreateReply::default().content(format!("No tournament found for the given ID {}. Try again with a different tournament ID.", tournament_id)).ephemeral(true)).await?;
            return Ok(());
        }
    };

    if tournament.status != TournamentStatus::Paused {
        ctx.send(
            CreateReply::default()
                .content(format!(
                    "The tournament with ID {} is not currently paused.\n\nTournament status: {}",
                    tournament_id, tournament.status
                ))
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    ctx.data()
        .database
        .set_tournament_status(tournament_id, TournamentStatus::Started)
        .await?;

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Successfully unpaused tournament with ID {}",
                tournament_id
            ))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Disqualify a player from a given tournament.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
async fn disqualify(ctx: BotContext<'_>, tournament_id: i32, player: User) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let tournament = match ctx
        .data()
        .database
        .get_tournament(&guild_id, tournament_id)
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
        .get_match_by_player(tournament.tournament_id, &player.id.to_string())
        .await?
    {
        Some(bracket) => bracket,
        None => {
            ctx.send(
                CreateReply::default()
                    .content(format!(
                        "An unfinished match could not be found for <@{}> in tournament {}.",
                        player.id, tournament_id
                    ))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let opponent = bracket.get_opponent(&player.id.to_string())?;
    ctx.data()
        .database
        .set_winner(&bracket.match_id, &opponent.discord_id)
        .await?;

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Successfully disqualified <@{}> from match {}",
                player.id, bracket.match_id
            ))
            .ephemeral(true),
    )
    .await?;
    let description = format!(
        r#"Player <@{player_id}> was disqualified from the tournament.
Match ID: {match_id}.
Tournament ID: {tournament_id}.
Tournament name: {tournament_name}.
Disqualified by: {disqualified_by}."#,
        player_id = player.id,
        match_id = bracket.match_id,
        tournament_id = tournament.tournament_id,
        tournament_name = tournament.name,
        disqualified_by = ctx.author().name
    );
    ctx.log(
        "Player disqualified!",
        description,
        log::State::SUCCESS,
        log::Model::MARSHAL,
    )
    .await?;
    Ok(())
}

/// List all currently active tournaments.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
#[instrument]
async fn next_round(ctx: BotContext<'_>, tournament_id: i32) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default().embed(CreateEmbed::default().description("Running commands...")),
        )
        .await?;
    let guild_id = ctx.guild_id().unwrap().to_string();

    let tournament = match ctx
        .data()
        .database
        .get_tournament(&guild_id, tournament_id)
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
        .get_matches_by_tournament(tournament_id, Some(tournament.current_round))
        .await?;

    let (with_winners, without_winners): (Vec<Match>, Vec<Match>) = brackets
        .into_iter()
        .partition(|bracket| bracket.winner.is_some());

    if !without_winners.is_empty() {
        // TODO: Show unfinished matches as a table or a CSV file
        ctx.send(CreateReply::default().content("Unable to advance to the next round. Some players have not finished their matches yet!").ephemeral(true)).await?;
        return Ok(());
    }

    let round = tournament.current_round + 1;
    let next_round_brackets = generate_next_round(with_winners, round)?;
    let new_brackets_count = next_round_brackets.len();

    for mut bracket in next_round_brackets {
        ctx.data()
            .database
            .create_match(tournament_id, round, bracket.sequence_in_round)
            .await?;

        for _ in 0..bracket.match_players.len() {
            ctx.data()
                .database
                .enter_match(
                    &Match::generate_id(tournament_id, round, bracket.sequence_in_round),
                    &bracket.match_players.remove(0).discord_id,
                    PlayerType::Player,
                )
                .await?;
        }
    }

    ctx.data().database.next_round(tournament_id).await?;

    if ctx
        .confirmation(
            &msg,
            CreateEmbed::default().description("Do you want to select map for next round?"),
        )
        .await?
    {
        let mode = tournament.mode.clone();
        let map = ctx.map_selection(&msg, &mode).await?;
        ctx.data()
            .database
            .set_map(tournament_id, &(map.into()))
            .await?;
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
    let description = format!(
        r#"The tournament has advanced to round {}.
Tournament ID: {}.
Tournament name: {}.
Number of matches: {}.
Advanced by: {}."#,
        round,
        tournament.tournament_id,
        tournament.name,
        new_brackets_count,
        ctx.author().name
    );
    ctx.log(
        "Tournament advanced!",
        description,
        log::State::SUCCESS,
        log::Model::MARSHAL,
    )
    .await?;
    Ok(())
}

/// Generates the matches for the next round.
fn generate_next_round(brackets: Vec<Match>, round: i32) -> Result<Vec<Match>, BotError> {
    let mut next_round_brackets = Vec::with_capacity(brackets.len() / 2);
    let tournament_id = brackets[0].tournament_id.to_owned();
    let mut brackets_iter = brackets.into_iter();

    for _i in 1..=next_round_brackets.len() {
        let old_bracket_1 = brackets_iter.next().ok_or(anyhow!("Error advancing to the next round: Ran out of brackets from the previous round while generating the next round."))?;
        let old_bracket_2 = brackets_iter.next().ok_or(anyhow!("Error advancing to the next round: Ran out of brackets from the previous round while generating the next round."))?;

        let player_1 = old_bracket_1
            .get_winning_player()
            .ok_or(anyhow!(
                "Error advancing to the next round: Unable to find the winning player in Match {}",
                old_bracket_1.match_id
            ))?
            .to_owned();

        let player_2 = old_bracket_2
            .get_winning_player()
            .ok_or(anyhow!(
                "Error advancing to the next round: Unable to find the winning player in Match {}",
                old_bracket_2.match_id
            ))?
            .to_owned();

        let new_sequence = (old_bracket_1.sequence_in_round as f32 / 2.0).ceil() as i32;

        if new_sequence != (old_bracket_2.sequence_in_round / 2) {
            return Err(anyhow!("Error generating matches for the next round. Previous round matches do not match:\n\nMatch ID 1: {}\nMatch ID 2: {}", old_bracket_1.match_id, old_bracket_2.match_id));
        }

        next_round_brackets.push(Match::new(
            tournament_id,
            round,
            new_sequence,
            vec![player_1, player_2],
            "0-0",
        ))
    }

    Ok(next_round_brackets)
}
