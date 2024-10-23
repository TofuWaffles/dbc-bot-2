use super::{checks::is_marshal_or_higher, CommandsContainer};
use crate::commands::checks::is_manager;
use crate::database::models::{BrawlMap, Match, PlayerType, Tournament, TournamentStatus};
use crate::database::{BattleDatabase, Database, MatchDatabase, TournamentDatabase, UserDatabase};
use crate::utils::discord::{modal, select_channel, select_options, select_role};
use crate::utils::error::CommonError::*;
use crate::{
    log::{self, Log},
    utils::shorthand::BotContextExt,
    BotContext, BotData, BotError,
};
use anyhow::anyhow;
use chrono::DateTime;
use futures::StreamExt;
use poise::serenity_prelude::{
    CreateActionRow, CreateAttachment, CreateButton, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, Mentionable, ReactionType, UserId,
};
use poise::ReplyHandle;
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
            get_battle_logs(),
            set_map(),
            disqualify_slash(),
            marshal_menu(),
        ]
    }
}

/// Get information about a tournament.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
#[instrument]
async fn get_tournament(ctx: BotContext<'_>, tournament_id: i32) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;

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
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;

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
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;
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
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;
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
                    .get_player_active_tournaments(&guild_id, &player.id)
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
                    .get_match_by_player(player_active_tournaments[0].tournament_id, &player.id)
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
                                ("Tournament ID", bracket.tournament()?.to_string(), false),
                                ("Round", bracket.round()?.to_string(), false),
                                (
                                    "Player 1",
                                    format!(
                                        "{:#?}",
                                        bracket
                                            .match_players
                                            .first()
                                            .map(|p| format!("<@{}>", p.discord_id))
                                    ),
                                    false,
                                ),
                                (
                                    "Player 2",
                                    format!(
                                        "{:#?}",
                                        bracket
                                            .match_players
                                            .get(1)
                                            .map(|p| format!("<@{}>", p.discord_id))
                                    ),
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
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;

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
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;

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
#[poise::command(
    slash_command,
    guild_only,
    check = "is_marshal_or_higher",
    rename = "disqualify"
)]
async fn disqualify_slash(
    ctx: BotContext<'_>,
    tournament_id: i32,
    player: User,
) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;
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
    disqualify(&ctx, &tournament, player).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    guild_only,
    check = "is_marshal_or_higher",
    context_menu_command = "Disqualify current round"
)]
async fn disqualify_context(ctx: BotContext<'_>, player: User) -> Result<(), BotError> {
    let tournament_id = match ctx
        .data()
        .database
        .get_tournament_id_by_player(&player.id)
        .await?
    {
        Some(t) => t,
        None => {
            ctx.send(
                CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .title("Player not found in any tournament")
                            .description(format!("Player <@{}> is not currently in any tournament. Please try again with a different player.", player.id)),
                    )
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;
    let t = match ctx
        .data()
        .database
        .get_tournament(&guild_id, tournament_id)
        .await?
    {
        Some(t) => t,
        None => {
            ctx.send(
                CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .title("Tournament not found")
                            .description(format!("Tournament with ID {} was not found. Please try again with a different tournament ID.", tournament_id)),
                    )
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };
    disqualify(&ctx, &t, player).await
}

async fn disqualify(
    ctx: &BotContext<'_>,
    tournament: &Tournament,
    player: User,
) -> Result<(), BotError> {
    let bracket = match ctx
        .data()
        .database
        .get_match_by_player(tournament.tournament_id, &player.id)
        .await?
    {
        Some(bracket) => bracket,
        None => {
            ctx.send(
                CreateReply::default()
                    .content(format!(
                        "An unfinished match could not be found for <@{}> in tournament {}.",
                        player.id, tournament.tournament_id
                    ))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let opponent = bracket
        .get_opponent(&player.id.to_string())?
        .to_user(ctx)
        .await?;
    ctx.data()
        .database
        .set_winner(&bracket.match_id, &opponent.id, "n/a")
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

/// Progress the tournament into the next round
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
#[instrument]
async fn next_round(ctx: BotContext<'_>, tournament_id: i32) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default().embed(CreateEmbed::default().description("Running commands...")),
        )
        .await?;
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;

    let tournament = match ctx
        .data()
        .database
        .get_tournament(&guild_id, tournament_id)
        .await?
    {
        Some(tournament) => tournament,
        None => {
            ctx.prompt(
                &msg,
                CreateEmbed::new()
                    .title("No ID was given")
                    .description("Try again with an existing tournament ID."),
                None,
            )
            .await?;
            return Ok(());
        }
    };

    next_round_helper(&ctx, &msg, &tournament).await
}

/// Helper function to progress the tournament to the next round.
///
/// Will automatically fail if the tournament is on its final round or if it's no longer active.
async fn next_round_helper(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    tournament: &Tournament,
) -> Result<(), BotError> {
    type ConditionFn = fn(&Tournament) -> bool;
    type Condition<'a> = (ConditionFn, &'a str, &'a str);
    let conditions: Vec<Condition> = vec![
        (|t| t.status != TournamentStatus::Started, "Non active tournament!","This tournament is not currently active. Please try again when the tournament is active again."),
        (|t| t.current_round == t.rounds, "No more rounds!","Unable to advance to the next round. This tournament is currently on its final round.")
    ];

    for (predicate, title, message) in conditions {
        if predicate(tournament) {
            ctx.prompt(
                msg,
                CreateEmbed::new().title(title).description(message),
                None,
            )
            .await?;
            return Ok(());
        }
    }

    let brackets = ctx
        .data()
        .database
        .get_matches_by_tournament(tournament.tournament_id, Some(tournament.current_round))
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
    let mut next_round_brackets = generate_next_round(with_winners, round)?;
    println!("{:#?}", next_round_brackets);
    let new_brackets_count = next_round_brackets.len();
    while let Some(bracket) = next_round_brackets.pop() {
        ctx.data()
            .database
            .create_match(bracket.tournament()?, bracket.round()?, bracket.sequence()?)
            .await?;

        for player in bracket.match_players.iter() {
            let match_id = Match::generate_id(
                tournament.tournament_id,
                bracket.round()?,
                bracket.sequence()?,
            );
            let player = player.to_user(ctx).await?;
            ctx.data()
                .database
                .enter_match(&match_id, &player.id, PlayerType::Player)
                .await?;
        }
    }

    ctx.data()
        .database
        .set_current_round(tournament.tournament_id, tournament.current_round + 1)
        .await?;
    if ctx
        .confirmation(
            msg,
            CreateEmbed::default().description("Do you want to select map for next round?"),
        )
        .await?
    {
        let mode = tournament.mode;
        let map = ctx.map_selection(msg, &mode).await?;
        ctx.data()
            .database
            .set_map(tournament.tournament_id, &(map.into()))
            .await?;
    }

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Successfully advanced the tournament with ID {} to the round {}.",
                tournament.tournament_id, round
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
    let tournament_id = brackets[0].tournament()?;
    let mut brackets_iter = brackets.into_iter();

    for _i in 1..=next_round_brackets.len() {
        let prev_bracket_1 = brackets_iter.next().ok_or(anyhow!("Error advancing to the next round: Ran out of brackets from the previous round while generating the next round."))?;
        let prev_bracket_2 = brackets_iter.next().ok_or(anyhow!("Error advancing to the next round: Ran out of brackets from the previous round while generating the next round."))?;

        let player_1 = prev_bracket_1
            .get_winning_player()
            .ok_or(anyhow!(
                "Error advancing to the next round: Unable to find the winning player in Match {}",
                prev_bracket_1.match_id
            ))?
            .to_owned();

        let player_2 = prev_bracket_2
            .get_winning_player()
            .ok_or(anyhow!(
                "Error advancing to the next round: Unable to find the winning player in Match {}",
                prev_bracket_2.match_id
            ))?
            .to_owned();

        let cur_sequence = (prev_bracket_1.sequence()? + 1) >> 1;
        if cur_sequence != (prev_bracket_2.sequence()? + 1) >> 1 {
            return Err(anyhow!("Error generating matches for the next round. Previous round matches do not match:\n\nMatch ID 1: {}\nMatch ID 2: {}", prev_bracket_1.match_id, prev_bracket_2.match_id));
        }

        next_round_brackets.push(Match::new(
            tournament_id,
            round,
            cur_sequence,
            vec![player_1, player_2],
            "0-0",
        ))
    }
    Ok(next_round_brackets)
}

#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
async fn get_battle_logs(
    ctx: BotContext<'_>,
    #[description = "The player to get the battle log for"] player: User,
    #[description = "The match ID to get the battle log for"] match_id: Option<String>,
) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default()
                .content("Fetching the battle log...")
                .ephemeral(true),
        )
        .await?;
    async fn inner(
        ctx: &BotContext<'_>,
        msg: &ReplyHandle<'_>,
        player: &User,
        match_id: Option<String>,
    ) -> Result<(), BotError> {
        let current_match = match match_id {
            Some(mid) => ctx.data().database.get_match_by_id(&mid).await?,
            None => ctx.data().database.get_current_match(&player.id).await?,
        }
        .ok_or(anyhow!("Match not found for this player"))?;
        let record = ctx
            .data()
            .database
            .get_record(&current_match.match_id)
            .await?
            .ok_or(anyhow!("Record not found for this match"))?;
        let img = ctx
            .data()
            .apis
            .images
            .clone()
            .battle_log(ctx, record, current_match)
            .await?;
        let reply =
            { CreateReply::default().attachment(CreateAttachment::bytes(img, "battle_log.png")) };
        msg.edit(*ctx, reply).await?;
        Ok(())
    }
    if let Err(e) = inner(&ctx, &msg, &player, match_id.clone()).await {
        let embed = CreateEmbed::new()
            .title("An error encoutered!")
            .description(format!("{}", e));
        ctx.prompt(&msg, embed, None).await?;
    }
    Ok(())
}

#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
async fn update_map(ctx: BotContext<'_>) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default()
                .content("Fetching the map from Brawlify")
                .ephemeral(true),
        )
        .await?;
    let response = ctx.data().apis.brawlify.get_maps().await?;
    let data = response.handler(&ctx, &msg).await?;
    if let Some(maps) = data {
        let db = &ctx.data().database;
        let list = maps.list.to_owned();
        for map in list.into_iter() {
            let m: BrawlMap = map.into();
            db.add_map(&m).await?;
        }
        ctx.prompt(
            &msg,
            CreateEmbed::new()
                .title("Updated successfully")
                .description("All maps have been updated"),
            None,
        )
        .await?;
    }
    Ok(())
}

#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
async fn marshal_menu(ctx: BotContext<'_>) -> Result<(), BotError> {
    ctx.defer_ephemeral().await?;
    let msg = ctx
        .send(CreateReply::default().ephemeral(true).embed(
            CreateEmbed::new().description("Running the command that only Marshal can see..."),
        ))
        .await?;

    let guild = ctx.guild_id().ok_or(NotInAGuild)?;
    let tournaments = ctx.data().database.get_all_tournaments(&guild).await?;
    if tournaments.is_empty() {
        let embed = CreateEmbed::new()
            .title("No tournaments found")
            .description("There are no tournaments found in this server.");
        ctx.prompt(&msg, embed, None).await?;
        return Ok(());
    }
    let embed = {
        let fields: Vec<(String, String, bool)> = tournaments
            .iter()
            .map(|t| (t.name.to_string(), format!("{}", t.tournament_id), true))
            .collect();
        CreateEmbed::new()
            .title("Select a tournament")
            .description("Here are some of the tournaments that you can manage.")
            .fields(fields)
    };
    let tid = select_options(&ctx, &msg, embed, None, &tournaments).await?;
    let tournament = tournaments
        .into_iter()
        .find(|t| t.tournament_id.to_string() == tid)
        .unwrap();
    let buttons = {
        vec![
            CreateButton::new("properties")
                .label("Properties")
                .style(poise::serenity_prelude::ButtonStyle::Primary),
            CreateButton::new("players")
                .label("Players")
                .style(poise::serenity_prelude::ButtonStyle::Primary),
        ]
    };
    ctx.prompt(
        &msg,
        CreateEmbed::new()
            .title(format!("Tournament Menu for {}", tournament.name))
            .description("Below are the information about the tournament")
            .fields(vec![
                ("Tournament", tournament.name.clone(), true),
                ("ID", tournament.tournament_id.to_string(), true),
                ("Mode", tournament.mode.to_string(), true),
                ("Status", tournament.status.to_string(), true),
                (
                    "Round",
                    format!("{}/{}", tournament.current_round, tournament.rounds),
                    true,
                ),
                ("Wins required", tournament.wins_required.to_string(), true),
                ("Map", tournament.map.name.clone(), true),
            ]),
        buttons,
    )
    .await?;
    let mut ic = ctx.create_interaction_collector(&msg).await?;
    while let Some(interactions) = &ic.next().await {
        match interactions.data.custom_id.as_str() {
            "properties" => {
                interactions.defer(&ctx.http()).await?;
                tournament_property_page(&ctx, &msg, &tournament).await?;
            }
            "players" => {
                interactions.defer(&ctx.http()).await?;
                player_page(&ctx, &msg, &tournament).await?;
            }
            _ => {}
        }
    }
    Ok(())
}

async fn tournament_property_page(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    t: &Tournament,
) -> Result<(), BotError> {
    async fn update(ctx: &BotContext<'_>, t: &Tournament) -> Result<Tournament, BotError> {
        ctx.data()
            .database
            .get_tournament(&ctx.guild_id().ok_or(NotInAGuild)?, t.tournament_id)
            .await?
            .ok_or(TournamentNotExists(t.tournament_id.to_string()).into())
    }
    let mut t = t.clone();
    let manager = is_manager(*ctx).await?;
    let buttons = |t: &Tournament| {
        vec![
            CreateButton::new("pause")
                .label(if t.is_paused() { "Resume" } else { "Pause" })
                .emoji(ReactionType::Unicode(String::from(
                    ["⏸️", "▶️"][t.is_paused() as usize],
                )))
                .style(poise::serenity_prelude::ButtonStyle::Primary),
            CreateButton::new("win")
                .label("Win")
                .emoji(ReactionType::Unicode("🏆".to_string()))
                .style(poise::serenity_prelude::ButtonStyle::Primary),
            CreateButton::new("mode")
                .label("Mode")
                .emoji(ReactionType::Unicode("🎮".to_string()))
                .style(poise::serenity_prelude::ButtonStyle::Primary),
            CreateButton::new("map")
                .label("Map")
                .emoji(ReactionType::Unicode("🗺️".to_string()))
                .style(poise::serenity_prelude::ButtonStyle::Primary),
            CreateButton::new("advanced")
                .label("Advance")
                .emoji(ReactionType::Unicode("🛡️".to_string()))
                .style(poise::serenity_prelude::ButtonStyle::Primary)
                .disabled(!manager),
        ]
    };
    let embed = |t: &Tournament| {
        CreateEmbed::new()
            .title("Tournament configuration")
            .description(format!(
                "**These configurations are currently applied for {tname}`{tid}`**",
                tname = t.name,
                tid = t.tournament_id
            ))
            .fields(vec![
                ("Tournament status", t.status.to_string(), true),
                ("Wins required per round", t.wins_required.to_string(), true),
                (
                    "Current round",
                    format!("{}/{}", t.current_round, t.rounds),
                    true,
                ),
                ("Mode", t.mode.to_string(), true),
                ("Map", t.map.name.clone(), true),
            ])
    };
    ctx.prompt(msg, embed(&t), buttons(&t)).await?;
    let mut ic = ctx.create_interaction_collector(msg).await?;
    while let Some(interactions) = &ic.next().await {
        match interactions.data.custom_id.as_str() {
            "pause" => {
                interactions.defer(ctx.http()).await?;

                let status = if t.is_paused() {
                    TournamentStatus::Started
                } else {
                    TournamentStatus::Paused
                };
                ctx.data()
                    .database
                    .set_tournament_status(t.tournament_id, status)
                    .await?;
            }
            "win" => {
                #[derive(poise::Modal)]
                struct WinModal {
                    #[name = "Minimum wins required to win a match"]
                    #[placeholder = "Write the number of wins required to win a match here or leave it blank for 3!"]
                    wins_required: Option<String>,
                }
                let embed = CreateEmbed::new()
                    .title("Win requirement")
                    .description("Enter the number of wins required to win a match");
                let res = modal::<WinModal>(ctx, msg, embed).await?;
                let wins: i32 = res.wins_required.unwrap_or("3".to_string()).parse()?;
                ctx.data()
                    .database
                    .set_wins_required(&t.tournament_id, &wins)
                    .await?;
            }
            "mode" => {
                interactions.defer(ctx.http()).await?;
                let mode = ctx.mode_selection(msg).await?;
                ctx.data().database.set_mode(t.tournament_id, mode).await?;
            }
            "map" => {
                interactions.defer(ctx.http()).await?;
                let map = ctx.map_selection(msg, &t.mode).await?;
                ctx.data()
                    .database
                    .set_map(t.tournament_id, &map.into())
                    .await?;
            }
            "advanced" => {
                interactions.defer(ctx.http()).await?;
                return advance_page(ctx, msg, &t).await;
            }
            _ => {}
        }
        t = update(ctx, &t).await?;
        ctx.prompt(msg, embed(&t), buttons(&t)).await?;
    }
    Ok(())
}

async fn advance_page(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    t: &Tournament,
) -> Result<(), BotError> {
    let reply = {
        let embed = {
            let ac = t.announcement_channel(ctx).await?;
            let nc = t.notification_channel(ctx).await?;
            let pr = t.player_role(ctx).await?;
            CreateEmbed::new()
                .title("Tournament configuration")
                .description(format!(
                    r#"
**These configurations are applied for {tname}`{tid}`**
Choose one of the setting below if you want to carry out the changes!
Current configuration:
"#,
                    tname = t.name,
                    tid = t.tournament_id
                ))
                .fields(vec![
                    ("Announcement Channel", ac.mention().to_string(), true),
                    ("Notification Channel", nc.mention().to_string(), true),
                    ("Participant Role", pr.mention().to_string(), true),
                ])
        };
        let components = vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
            "select",
            CreateSelectMenuKind::String {
                options: vec![
                    CreateSelectMenuOption::new("Announcement Channel", "announcement"),
                    CreateSelectMenuOption::new("Notification Channel", "notification"),
                    CreateSelectMenuOption::new("Participant Role", "participant"),
                ],
            },
        ))];
        CreateReply::default()
            .ephemeral(true)
            .embed(embed)
            .components(components)
    };
    msg.edit(*ctx, reply.clone()).await?;
    let mut ic = ctx.create_interaction_collector(msg).await?;
    while let Some(interaction) = &ic.next().await {
        match interaction.data.custom_id.as_str() {
            "announcement" => {
                interaction.defer(ctx.http()).await?;
                let embed = CreateEmbed::new();
                select_channel(ctx, msg, embed).await?;
            }
            "notification" => {
                interaction.defer(ctx.http()).await?;
                let embed = CreateEmbed::new();
                select_channel(ctx, msg, embed).await?;
            }
            "participant" => {
                interaction.defer(ctx.http()).await?;
                let embed = CreateEmbed::new();
                select_role(ctx, msg, embed).await?;
            }
            _ => continue,
        }
        msg.edit(*ctx, reply.clone()).await?;
    }
    Ok(())
}

async fn player_page(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    t: &Tournament,
) -> Result<(), BotError> {
    let embed = {
        let players = t.count_players_in_current_round(ctx).await?;
        let finished = t.count_finished_matches(ctx).await?;
        CreateEmbed::new()
            .title(format!("Players' insight of {}", t.name))
            .description("")
            .fields(vec![
                ("Round", t.current_round.to_string(), true),
                ("Participants", format!("{}", players), true),
                (
                    "Finished",
                    format!("{}({}%)", finished, finished * 100 / (players >> 1)),
                    true,
                ),
            ])
    };
    let buttons = {
        vec![
            CreateButton::new("disqualify")
                .label("Disqualify")
                .style(poise::serenity_prelude::ButtonStyle::Primary),
            CreateButton::new("search")
                .label("Search")
                .style(poise::serenity_prelude::ButtonStyle::Primary),
        ]
    };
    ctx.prompt(msg, embed, buttons).await?;
    let mut ic = ctx.create_interaction_collector(msg).await?;
    while let Some(interaction) = &ic.next().await {
        match interaction.data.custom_id.as_str() {
            "disqualify" => {
                #[derive(poise::Modal)]
                struct DisqualifyModal {
                    #[name = "Player"]
                    player: Option<String>,
                }
                let res = modal::<DisqualifyModal>(
                    ctx,
                    msg,
                    CreateEmbed::new().title("Disqualify a player"),
                )
                .await?;
                let id = res.player.ok_or(anyhow!("No player was given"))?;
                let player = UserId::new(id.parse()?);
                disqualify(ctx, t, player.to_user(ctx).await?).await?;
            }
            "next" => {
                next_round_helper(ctx, msg, t).await?;
            }
            _ => {}
        }
    }
    Ok(())
}
