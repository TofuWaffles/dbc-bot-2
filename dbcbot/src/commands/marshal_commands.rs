use std::str::FromStr;

use super::user_commands::finish_match;
use super::{checks::is_marshal_or_higher, CommandsContainer};
use crate::commands::checks::is_manager;
use crate::commands::user_commands::display_user_profile_helper;
use crate::database::models::{
    BrawlMap, MatchPlayer, Player, PlayerType, Tournament, TournamentStatus,
};
use crate::database::{BattleDatabase, Database, MatchDatabase, TournamentDatabase, UserDatabase};
use crate::mail::MailDatabase;
use crate::utils::error::CommonError::*;
use crate::utils::shorthand::BotComponent;
use crate::utils::time::BattleDateTime;
use crate::{
    log::{self, Log},
    utils::shorthand::BotContextExt,
    BotContext, BotData, BotError,
};
use anyhow::anyhow;
use chrono::DateTime;
use futures::StreamExt;
use poise::serenity_prelude::{
    ButtonStyle, Color, CreateActionRow, CreateAttachment, CreateButton, CreateEmbedFooter,
    Mentionable, ReactionType, UserId,
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
            get_active_tournaments(),
            next_round(),
            get_match(),
            get_battle_logs(),
            set_map(),
            disqualify_slash(),
            disqualify_context(),
            list_matches(),
            list_players_slash(),
            marshal_menu(),
            add_map_slash(),
            remove_user(),
            unban(),
            user_profile(),
            view_match_context(),
            extract_conversation(),
        ]
    }
}

#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
#[instrument]
async fn get_player(ctx: BotContext<'_>, user: UserId) -> Result<(), BotError> {
    let player = ctx.data().database.get_player_by_discord_id(&user).await?;

    match player {
        Some(player) => {
            ctx.send(
                CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .title(format!("Here is all for info for {}", player.discord_name))
                            .fields(vec![
                                ("In-Game Name", player.player_name, true),
                                ("Game Tag", player.player_tag, true),
                                ("Discord ID", player.discord_id, true),
                                ("Discord Name", player.discord_name, true),
                                ("Trophies", player.trophies.to_string(), false),
                            ]),
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
            warn!("Player with Discord ID {} not found", user.get());
        }
    };

    Ok(())
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
                    .embed(CreateEmbed::new().title(tournament.name).fields(vec![
                            ("ID", tournament.tournament_id.to_string(), true),
                            ("Status", tournament.status.to_string(), true),
                            ("Rounds", tournament.rounds.to_string(), true),
                            ("Current Round", tournament.current_round.to_string(), true),
                            (
                                "Wins Required Per Round",
                                tournament.wins_required.to_string(),
                                true,
                            ),
                            ("Map", format!("{:#?}", tournament.map), true),
                            (
                                "Created At",
                                DateTime::from_timestamp(tournament.created_at, 0)
                                    .unwrap_or_default()
                                    .to_rfc2822(),
                                true,
                            ),
                            ("Started At", start_time_str, true),
                        ]))
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
async fn get_active_tournaments(ctx: BotContext<'_>) -> Result<(), BotError> {
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
        .send(
            CreateReply::default()
                .embed(CreateEmbed::default().description("Loading maps..."))
                .ephemeral(true),
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
    let map = ctx.components().map_selection(&msg, &mode).await?;
    ctx.data()
        .database
        .set_map(tournament_id, &map.clone().into())
        .await?;

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Successfully set the map of tournament {} to {}",
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
    let log = ctx.build_log(
        "Map set successfully!",
        description,
        log::State::SUCCESS,
        log::Model::TOURNAMENT,
    );
    ctx.log(log, None).await?;
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
        Some(mut bracket) => {
            let player_2 = bracket.match_players.pop();
            let player_1 = bracket.match_players.pop();

            let mut fields = vec![
                ("Tournament ID", bracket.tournament()?.to_string(), true),
                ("Round", bracket.round()?.to_string(), true),
                (
                    "Winner",
                    match bracket.winner {
                        Some(w) => format!("<@{}>", w),
                        None => "None".to_string(),
                    },
                    true,
                ),
            ];
            let player_1 = match player_1 {
                Some(p) => {
                    fields.push(("Player 1", format!("<@{}>", p.discord_id), true));
                    fields.push((
                        "Player 1 Ready",
                        if p.ready {
                            "Ready for battle!".to_string()
                        } else {
                            "Not ready yet!".to_string()
                        },
                        false,
                    ));
                    ctx.data()
                        .database
                        .get_player_by_discord_id(&UserId::from_str(&p.discord_id)?)
                        .await?
                        .ok_or(anyhow!(
                            "Unable to find player {} in database",
                            p.discord_id
                        ))?
                }
                None => {
                    fields.push(("Player 1", "None".to_string(), true));
                    let mut p = Player::default();
                    p.player_name = "None".to_string();
                    p.discord_name = "None".to_string();
                    p
                }
            };
            let player_2 = match player_2 {
                Some(p) => {
                    fields.push(("Player 2", format!("<@{}>", p.discord_id), true));
                    fields.push((
                        "Player 2 Ready",
                        if p.ready {
                            "Ready for battle!".to_string()
                        } else {
                            "Not ready yet!".to_string()
                        },
                        false,
                    ));
                    ctx.data()
                        .database
                        .get_player_by_discord_id(&UserId::from_str(&p.discord_id)?)
                        .await?
                        .ok_or(anyhow!(
                            "Unable to find player {} in database",
                            p.discord_id
                        ))?
                }
                None => {
                    fields.push(("Player 2", "None".to_string(), true));
                    let mut p = Player::default();
                    p.player_name = "None".to_string();
                    p.discord_name = "None".to_string();
                    p
                }
            };

            let images = ctx
                .data()
                .apis
                .images
                .match_image(&player_1, &player_2)
                .await?;
            ctx.send(
                CreateReply::default()
                    .content("Here is your match information:")
                    .embed(
                        CreateEmbed::new()
                            .title(format!("Match {}", bracket.match_id))
                            .fields(fields),
                    )
                    .attachment(CreateAttachment::bytes(
                        images,
                        format!("match_{}.png", bracket.match_id.replace(".", "-")),
                    ))
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
    reason: Option<String>,
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
    let msg = &ctx
        .send(
            CreateReply::default()
                .content("Loading...")
                .ephemeral(true)
                .reply(true),
        )
        .await?;
    disqualify(&ctx, &msg, &tournament, &player, reason).await?;
    Ok(())
}

#[poise::command(
    guild_only,
    check = "is_marshal_or_higher",
    context_menu_command = "Disqualify Player"
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
    let embed = CreateEmbed::new()
        .title(format!("Disqualify player: {}?", player.mention()))
        .description(format!(
            "Are you sure you want to disqualify <@{}> from the tournament? Provide a reason (leave blank for none).",
            player.id
        ))
        .color(Color::RED);
    let buttons = {
        vec![CreateButton::new("reason")
            .label("Specify Reason")
            .style(poise::serenity_prelude::ButtonStyle::Primary)]
    };
    let msg = &ctx
        .send(CreateReply::default().content("Loading...").ephemeral(true))
        .await?;
    ctx.components().prompt(msg, embed, buttons).await?;
    let mut ic = ctx.create_interaction_collector(msg).await?;
    while let Some(interaction) = &ic.next().await {
        match interaction.data.custom_id.as_str() {
            "reason" => {
                #[derive(poise::Modal)]
                #[name = "Disqualify Player Reason"]
                struct DisqualifyModal {
                    #[name = "Reason"]
                    #[placeholder = "Enter reason for disqualification here"]
                    #[paragraph]
                    reason: Option<String>,
                }
                let res = ctx
                    .components()
                    .modal::<DisqualifyModal>(
                        msg,
                        CreateEmbed::new()
                            .title(format!("Disqualify player: {}?", player.mention())),
                    )
                    .await?;
                let reason = res.reason.map(|r| r.trim().to_string());
                disqualify(&ctx, &msg, &t, &player, reason).await?;
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn disqualify(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    tournament: &Tournament,
    user: &User,
    reason: Option<String>,
) -> Result<(), BotError> {
    let embed = CreateEmbed::new()
        .title(format!("Disqualify player: {}?", user.name))
        .description(format!(
            "Are you sure you want to disqualify {} from the tournament with the following reason: {}",
            user.mention(), reason.clone().unwrap_or("None".to_string())
        ))
        .color(Color::RED);
    let decision = ctx.components().confirmation(&msg, embed).await?;
    if !decision {
        let embed = CreateEmbed::new()
            .title("Disqualification cancelled")
            .description("Operation is cancelled. No action was taken.")
            .color(Color::DARK_GREEN);
        return ctx.components().prompt(&msg, embed, None).await;
    }
    let bracket = match ctx
        .data()
        .database
        .get_match_by_player(tournament.tournament_id, &user.id)
        .await?
    {
        Some(bracket) => bracket,
        None => {
            ctx.send(
                CreateReply::default()
                    .content(format!(
                        "An unfinished match could not be found for <@{}> in tournament {}.",
                        user.id, tournament.tournament_id
                    ))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    if bracket.winner.is_some() {
        ctx.send(
            CreateReply::default()
                .content(format!(
                    "Unable to disqualify player {}. Their furthest match {} has already ended.",
                    user.mention(),
                    bracket.match_id
                ))
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    if bracket.match_players.len() < 2 {
        ctx.send(
            CreateReply::default()
                .content(format!(
                    "Unable to disqualify player {}. They do not have an opponent in their match ({}) yet.",
                    user.mention(),
                    bracket.match_id
                ))
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    let score = "WON-DISQUALIFIED";

    let opponent = ctx
        .data()
        .database
        .get_player_by_discord_id(&bracket.get_opponent(&user.id.to_string())?.user_id()?)
        .await?
        .ok_or(anyhow!(
            "Unable to find opponent for player {} in match {}",
            ctx.author().id,
            bracket.match_id
        ))?;
    finish_match(ctx, tournament, &bracket, &opponent, score).await?;
    let embed = CreateEmbed::new()
        .title("Disqualification successful")
        .description(format!(
            "Player {} has been disqualified from the tournament with the following reason: {}",
            user.mention(),
            reason.clone().unwrap_or("None".to_string())
        ));
    ctx.components().prompt(msg, embed, None).await?;
    let fields = vec![
        ("Tournament ID", tournament.tournament_id.to_string(), true),
        ("Tournament name", tournament.name.to_string(), true),
        ("Match ID", bracket.match_id.to_string(), true),
        ("Disqualified player", user.name.to_string(), true),
        ("Disqualified by", ctx.author().name.to_string(), true),
        (
            "Reason",
            match reason {
                Some(r) => r,
                None => "None".to_string(),
            },
            true,
        ),
    ];
    let log = ctx
        .build_log(
            "Player disqualified!",
            format!(
                "Player {} was disqualified from the tournament",
                user.mention()
            ),
            log::State::SUCCESS,
            log::Model::MARSHAL,
        )
        .fields(fields);
    ctx.log(log, None).await?;

    Ok(())
}

/// List all players in a tournament in csv format.
#[poise::command(
    slash_command,
    guild_only,
    check = "is_marshal_or_higher",
    rename = "list_players"
)]
#[instrument]
async fn list_players_slash(
    ctx: BotContext<'_>,
    tournament_id: i32,
    round: Option<i32>,
) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default()
                .embed(
                    CreateEmbed::default()
                        .title("Running command...")
                        .description("Fetching the players for you.")
                        .footer(CreateEmbedFooter::new("This may take a while")),
                )
                .ephemeral(true)
                .reply(true),
        )
        .await?;
    let tournament = match ctx
        .data()
        .database
        .get_tournament(&ctx.guild_id().unwrap(), tournament_id)
        .await?
    {
        Some(t) => t,
        None => {
            ctx.send(
                CreateReply::default()
                    .content(format!("No tournament found for ID {}", tournament_id))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };
    list_players(&ctx, &msg, &tournament, round).await
}

async fn list_players(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    tournament: &Tournament,
    round: Option<i32>,
) -> Result<(), BotError> {
    let tournament_id = tournament.tournament_id;
    let (msg_content, filename) = match round {
        Some(r) => {
            if r > tournament.rounds {
                ctx.components()
                    .prompt(
                        msg,
                        CreateEmbed::new()
                            .title("Invalid round number")
                            .description(format!(
                                "Tournament {} only has {} rounds!. You entered: {} rounds",
                                tournament_id, tournament.rounds, r
                            )),
                        None,
                    )
                    .await?;
                return Ok(());
            }
            (
                format!(
                    "Here are all the players in round {} of tournament {} (ID: {})",
                    r, tournament.name, tournament_id
                ),
                format!("players_tournament_{}_round_{}.csv", tournament_id, r),
            )
        }
        None => (
            format!(
                "Here are all the players in tournament {} (ID: {})",
                tournament.name, tournament_id
            ),
            format!("players_tournament_{}.csv", tournament_id),
        ),
    };

    let players = ctx
        .data()
        .database
        .get_tournament_players(tournament_id, round)
        .await?
        .into_iter()
        .filter(|p| !p.deleted)
        .collect::<Vec<Player>>();

    let mut csv_str = "Discord Name,Discord ID,In-Game Name,Player Tag\n".to_string();

    for player in players {
        csv_str.push_str(&format!(
            "{},{},{},{}\n",
            player.discord_name, player.discord_id, player.player_name, player.player_tag,
        ));
    }
    msg.delete(*ctx).await?;
    ctx.send(
        CreateReply::default()
            .content(msg_content)
            .attachment(CreateAttachment::bytes(csv_str.as_bytes(), filename))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
#[instrument]
async fn list_matches(
    ctx: BotContext<'_>,
    tournament_id: i32,
    round: Option<i32>,
) -> Result<(), BotError> {
    let tournament = match ctx
        .data()
        .database
        .get_tournament(&ctx.guild_id().unwrap(), tournament_id)
        .await?
    {
        Some(t) => t,
        None => {
            ctx.send(
                CreateReply::default()
                    .content(format!("No tournament found for ID {}", tournament_id))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let (msg_content, filename) = match round {
        Some(r) => {
            if r > tournament.rounds {
                ctx.send(
                    CreateReply::default()
                        .content(format!(
                            "Tournament {} only has {} rounds!. You entered: {} rounds",
                            tournament_id, tournament.rounds, r
                        ))
                        .ephemeral(true),
                )
                .await?;
                return Ok(());
            }
            (
                format!(
                    "Here are all the players in round {} of tournament {} (ID: {})",
                    r, tournament.name, tournament_id
                ),
                format!("matches_tournament_{}_round_{}.csv", tournament_id, r),
            )
        }
        None => (
            format!(
                "Here are all the players in tournament {} (ID: {})",
                tournament.name, tournament_id
            ),
            format!("matches_tournament_{}.csv", tournament_id),
        ),
    };

    let matches = ctx
        .data()
        .database
        .get_matches_by_tournament(tournament_id, round)
        .await?;

    let mut csv_str = "Match ID,Player 1,Player 2,Score,Winner\n".to_string();

    let empty_player = MatchPlayer {
        match_id: "".to_string(),
        discord_id: "No Player".to_string(),
        player_type: PlayerType::Dummy,
        ready: false,
    };

    for bracket in matches {
        csv_str.push_str(&format!(
            "{},{},{},{},{}\n",
            bracket.match_id,
            bracket
                .match_players
                .first()
                .unwrap_or(&empty_player)
                .discord_id,
            bracket
                .match_players
                .get(1)
                .unwrap_or(&empty_player)
                .discord_id,
            bracket.score,
            bracket.winner.unwrap_or("TBD".to_string())
        ));
    }

    ctx.send(
        CreateReply::default()
            .content(msg_content)
            .attachment(CreateAttachment::bytes(csv_str.as_bytes(), filename))
            .ephemeral(true),
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
            ctx.components()
                .prompt(
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
        (|t| t.status != TournamentStatus::Started, "No active tournament!","This tournament is not currently active. Please try again when the tournament is active again."),
        (|t| t.current_round == t.rounds, "No more rounds!","Unable to advance to the next round. This tournament is currently on its final round.")
    ];

    for (predicate, title, message) in conditions {
        if predicate(tournament) {
            ctx.components()
                .prompt(
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

    if brackets.iter().any(|b| b.winner.is_none()) {
        ctx.send(
            CreateReply::default()
                .content("Unable to proceed to the next round: Not all matches in the current round have finished.\n\nYou can run /list_matches <round> to view all the matches for the current round.")
                .ephemeral(true),
        ).await?;

        return Ok(());
    }

    let new_round = tournament.current_round + 1;
    ctx.data()
        .database
        .set_current_round(tournament.tournament_id, new_round)
        .await?;
    if ctx
        .components()
        .confirmation(
            msg,
            CreateEmbed::default().description("Do you want to select map for next round?"),
        )
        .await?
    {
        let mode = tournament.mode;
        let map = ctx.components().map_selection(msg, &mode).await?;
        ctx.data()
            .database
            .set_map(tournament.tournament_id, &(map.into()))
            .await?;
    }

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Successfully advanced the tournament with ID {} to the round {}.",
                tournament.tournament_id, new_round
            ))
            .ephemeral(true),
    )
    .await?;
    let description = format!(r#"The tournament has advanced to round {}"#, new_round,);
    let fields = vec![
        ("Tournament ID", tournament.tournament_id.to_string(), true),
        ("Tournament name", tournament.name.to_string(), true),
        ("Number of matches", (brackets.len() >> 1).to_string(), true),
        ("Advanced by", ctx.author().name.to_string(), true),
    ];
    let log = ctx
        .build_log(
            "Tournament advanced!",
            description,
            log::State::SUCCESS,
            log::Model::MARSHAL,
        )
        .fields(fields);
    ctx.log(log, None).await?;
    Ok(())
}

#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
async fn get_battle_logs(
    ctx: BotContext<'_>,
    #[description = "The match ID to get the battle log for"] match_id: String,
) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default().reply(true).ephemeral(true).embed(
                CreateEmbed::new()
                    .title("Fetching the battle log...")
                    .description("Hold on a second, we're fetching the battle log for you."),
            ),
        )
        .await?;
    async fn inner(
        ctx: &BotContext<'_>,
        msg: &ReplyHandle<'_>,
        match_id: String,
    ) -> Result<(), BotError> {
        let current_match = ctx
            .data()
            .database
            .get_match_by_id(&match_id)
            .await?
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
            .battle_log(ctx, record, &current_match)
            .await?;
        let reply = {
            let attachment = CreateAttachment::bytes(img, "battle_log.png");
            let embed = CreateEmbed::new()
                .title("Battle Log")
                .description(format!("Battle log for match {}", current_match.match_id));
            CreateReply::default()
                .attachment(attachment)
                .embed(embed)
                .ephemeral(true)
                .reply(true)
        };
        ctx.send(reply).await?;
        msg.delete(*ctx).await?;
        Ok(())
    }
    if let Err(e) = inner(&ctx, &msg, match_id).await {
        let embed = CreateEmbed::new()
            .title("An error encountered!")
            .description(format!("{}", e));
        ctx.components().prompt(&msg, embed, None).await?;
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
        ctx.components()
            .prompt(
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
        ctx.components().prompt(&msg, embed, None).await?;
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
    let tid = ctx
        .components()
        .select_options(&msg, embed, None, &tournaments)
        .await?;
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
            CreateButton::new("utilities")
                .label("Utilities")
                .style(poise::serenity_prelude::ButtonStyle::Primary),
        ]
    };
    ctx.components()
        .prompt(
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
            "utilities" => {
                interactions.defer(&ctx.http()).await?;
                utilities_page(&ctx, &msg, &tournament).await?;
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
    ctx.components().prompt(msg, embed(&t), buttons(&t)).await?;
    let mut ic = ctx.create_interaction_collector(msg).await?;
    while let Some(interactions) = &ic.next().await {
        match interactions.data.custom_id.as_str() {
            "pause" => {
                interactions
                    .create_response(
                        ctx,
                        poise::serenity_prelude::CreateInteractionResponse::Acknowledge,
                    )
                    .await?;

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
                let res = ctx.components().modal::<WinModal>(msg, embed).await?;
                let wins: i32 = res.wins_required.unwrap_or("3".to_string()).parse()?;
                ctx.data()
                    .database
                    .set_wins_required(&t.tournament_id, &wins)
                    .await?;
            }
            "mode" => {
                interactions
                    .create_response(
                        ctx,
                        poise::serenity_prelude::CreateInteractionResponse::Acknowledge,
                    )
                    .await?;
                let mode = ctx.components().mode_selection(msg).await?;
                ctx.default_map(t.tournament_id).await?;
                ctx.data().database.set_mode(t.tournament_id, mode).await?;
            }
            "map" => {
                interactions
                    .create_response(
                        ctx,
                        poise::serenity_prelude::CreateInteractionResponse::Acknowledge,
                    )
                    .await?;
                let map = ctx.components().map_selection(msg, &t.mode).await?;
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
        t.update(ctx).await?;
        ctx.components().prompt(msg, embed(&t), buttons(&t)).await?;
    }
    Ok(())
}

async fn advance_page(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    t: &Tournament,
) -> Result<(), BotError> {
    let mut t = t.clone();
    let buttons = vec![
        CreateButton::new("announcement")
            .label("Announcement Channel")
            .emoji(ReactionType::Unicode("🏆".to_string()))
            .style(poise::serenity_prelude::ButtonStyle::Primary),
        CreateButton::new("notification")
            .label("Notification Channel")
            .emoji(ReactionType::Unicode("🎮".to_string()))
            .style(poise::serenity_prelude::ButtonStyle::Primary),
        CreateButton::new("participant")
            .label("Participant Role")
            .emoji(ReactionType::Unicode("🗺️".to_string()))
            .style(poise::serenity_prelude::ButtonStyle::Primary),
    ];

    async fn embed(ctx: &BotContext<'_>, t: &Tournament) -> Result<CreateEmbed, BotError> {
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
                    if let Some(role) = pr {
                        ("Participant Role", role.mention().to_string(), true)
                    } else {
                        ("Participant Role", "None".to_string(), true)
                    },
                ])
        };
        Ok(embed)
    }

    ctx.components()
        .prompt(msg, embed(ctx, &t).await?, buttons.clone())
        .await?;
    let mut ic = ctx.create_interaction_collector(msg).await?;
    while let Some(interaction) = &ic.next().await {
        match interaction.data.custom_id.as_str() {
            "announcement" => {
                interaction
                    .create_response(
                        ctx,
                        poise::serenity_prelude::CreateInteractionResponse::Acknowledge,
                    )
                    .await?;
                let embed = CreateEmbed::new()
                    .title("Announcement Channel")
                    .description("Select the channel where the result will be sent");
                let channel = ctx.components().select_channel(msg, embed).await?;
                t.set_announcement_channel(ctx, &channel).await?;
            }
            "notification" => {
                interaction
                    .create_response(
                        ctx,
                        poise::serenity_prelude::CreateInteractionResponse::Acknowledge,
                    )
                    .await?;
                let embed = CreateEmbed::new()
                    .title("Notification Channel")
                    .description("Select the channel where the notification will be sent");
                let channel = ctx.components().select_channel(msg, embed).await?;
                t.set_notification_channel(ctx, &channel).await?;
            }
            "participant" => {
                interaction
                    .create_response(
                        ctx,
                        poise::serenity_prelude::CreateInteractionResponse::Acknowledge,
                    )
                    .await?;
                let embed = CreateEmbed::new()
                    .title("Participant Role")
                    .description("Select the role that will be assigned to the participants");
                let role = ctx.components().select_role(msg, embed).await?;
                t.set_player_role(ctx, &role).await?;
            }
            _ => continue,
        }
        t.update(ctx).await?;
        ctx.components()
            .prompt(msg, embed(ctx, &t).await?, buttons.clone())
            .await?;
    }
    Ok(())
}

async fn player_page(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    t: &Tournament,
) -> Result<(), BotError> {
    let embed = {
        struct DisplayData {
            name: String,
            current_round: String,
            count: String,
            finished: String,
            rounds: String,
            status: String,
        }

        let data = match t.status.clone() {
            TournamentStatus::Paused | TournamentStatus::Started => DisplayData {
                name: t.name.clone(),
                current_round: t.current_round.to_string(),
                count: format!(
                    "There are {} players in this current round",
                    t.count_players_in_current_round(&ctx).await?
                ),
                finished: format!("{}", t.count_finished_matches(&ctx).await?),
                rounds: t.rounds.to_string(),
                status: format!("{}", if t.is_paused() { "Paused" } else { "Started" }),
            },
            TournamentStatus::Pending => DisplayData {
                name: t.name.clone(),
                current_round: "Not available.".to_string(),
                count: format!(
                    "There are {} players in this tournament",
                    t.clone().count_registers(&ctx).await?
                ),
                finished: format!("Not available."),
                rounds: "Not available.".to_string(),
                status: t.status.to_string(),
            },
            TournamentStatus::Inactive => DisplayData {
                name: t.name.clone(),
                current_round: "The game is finished!".to_string(),
                count: format!(
                    "There are {} players in this tournament",
                    t.clone().count_registers(&ctx).await?
                ),
                finished: format!("What do you think?"),
                rounds: t.rounds.to_string(),
                status: t.status.to_string(),
            },
        };
        let DisplayData {
            name,
            current_round,
            count,
            finished,
            rounds,
            status,
        } = data;
        CreateEmbed::new()
            .title(format!("Players' insight of {}", name))
            .description("")
            .fields(vec![
                ("Status", status, true),
                ("Round", format!("{}/{}", current_round, rounds), true),
                ("Participants", format!("{}", count), true),
                ("Finished matches", format!("{}", finished), true),
            ])
    };
    let buttons = {
        vec![
            CreateButton::new("disqualify")
                .label("Disqualify")
                .style(poise::serenity_prelude::ButtonStyle::Primary),
            CreateButton::new("next")
                .label("Next Round")
                .style(poise::serenity_prelude::ButtonStyle::Primary),
            CreateButton::new("mail")
                .label("Extract Mail")
                .emoji(ReactionType::Unicode("👀".to_string()))
                .style(poise::serenity_prelude::ButtonStyle::Primary),
            CreateButton::new("players")
                .label("Player List")
                .style(poise::serenity_prelude::ButtonStyle::Primary),
        ]
    };
    ctx.components().prompt(msg, embed, buttons).await?;
    let mut ic = ctx.create_interaction_collector(msg).await?;
    while let Some(interaction) = &ic.next().await {
        match interaction.data.custom_id.as_str() {
            "disqualify" => {
                #[derive(poise::Modal)]
                #[name = "Disqualify Player"]
                struct DisqualifyModal {
                    #[name = "Player"]
                    #[placeholder = "Enter the player's Discord ID here"]
                    player: String,
                    #[name = "Reason"]
                    #[placeholder = "Enter reason for disqualification here"]
                    #[paragraph]
                    reason: Option<String>,
                }
                let res = ctx
                    .components()
                    .modal::<DisqualifyModal>(msg, CreateEmbed::new().title("Disqualify a player"))
                    .await?;
                let id = res.player;
                let reason = res.reason;
                let player = UserId::from_str(&id)?;
                disqualify(ctx, msg, t, &player.to_user(ctx).await?, reason).await?;
            }
            "next" => {
                interaction
                    .create_response(
                        ctx,
                        poise::serenity_prelude::CreateInteractionResponse::Acknowledge,
                    )
                    .await?;
                next_round_helper(ctx, msg, t).await?;
            }
            "players" => {
                #[derive(poise::Modal)]
                struct RoundModal {
                    #[name = "Round. Leave blank for current round"]
                    round: Option<String>,
                }
                let res = ctx
                    .components()
                    .modal::<RoundModal>(
                        msg,
                        CreateEmbed::new()
                            .title("Round selection")
                            .description("Press continue to proceed"),
                    )
                    .await?;
                let round: Option<i32> = res.round.and_then(|s| s.parse::<i32>().ok());

                list_players(ctx, msg, t, round).await?;
            }
            "mail" => {
                #[derive(poise::Modal)]
                struct GetPlayer {
                    #[name = "First Player"]
                    #[placeholder = "Enter the player's Discord ID here"]
                    first: String,
                    #[name = "Second Player"]
                    #[placeholder = "Enter the player's Discord ID here"]
                    second: String,
                }
                let res = ctx
                    .components()
                    .modal::<GetPlayer>(
                        msg,
                        CreateEmbed::new().title("Player selection").description(
                            "Please prepare Discord Id of 2 players to extract their conversation",
                        ),
                    )
                    .await?;

                let first = match UserId::from_str(&res.first.trim()) {
                    Ok(u) => match u.to_user(ctx).await {
                        Ok(u) => u,
                        Err(_) => {
                            ctx.components()
                                .prompt(
                                    msg,
                                    CreateEmbed::new().description("Invalid Discord ID"),
                                    None,
                                )
                                .await?;
                            return Ok(());
                        }
                    },
                    Err(_) => {
                        ctx.components()
                            .prompt(
                                msg,
                                CreateEmbed::new().description("Invalid Discord ID"),
                                None,
                            )
                            .await?;
                        return Ok(());
                    }
                };
                let second = match UserId::from_str(&res.second.trim()) {
                    Ok(u) => match u.to_user(ctx).await {
                        Ok(u) => u,
                        Err(_) => {
                            ctx.components()
                                .prompt(
                                    msg,
                                    CreateEmbed::new().description("Invalid Discord ID"),
                                    None,
                                )
                                .await?;
                            return Ok(());
                        }
                    },
                    Err(_) => {
                        ctx.components()
                            .prompt(
                                msg,
                                CreateEmbed::new().description("Invalid Discord ID"),
                                None,
                            )
                            .await?;
                        return Ok(());
                    }
                };
                return extract_convo_helper(ctx, msg, &first, &second).await;
            }
            _ => {}
        }
    }
    Ok(())
}

async fn utilities_page(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    _tournament: &Tournament,
) -> Result<(), BotError> {
    let embed = CreateEmbed::default()
            .title("Utilities")
            .description("Tournament utilities includes\n-Add map: Update the latest map from Brawlify to the database")
            .footer(CreateEmbedFooter::new("This may take a while."));
    let buttons = {
        vec![CreateButton::new("add_map")
            .label("Map update")
            .style(poise::serenity_prelude::ButtonStyle::Primary)]
    };
    ctx.components().prompt(msg, embed, buttons).await?;
    let mut ic = ctx.create_interaction_collector(msg).await?;
    if let Some(interactions) = &ic.next().await {
        if interactions.data.custom_id.as_str() == "add_map" {
            add_maps(*ctx, msg).await?;
        }
    }

    Ok(())
}

pub async fn add_maps(ctx: BotContext<'_>, msg: &ReplyHandle<'_>) -> Result<(), BotError> {
    let embed = CreateEmbed::default()
        .title("Adding maps to the database")
        .description("This command will add all maps to the database.")
        .footer(CreateEmbedFooter::new("This may take a while."));
    ctx.components().prompt(msg, embed, None).await?;
    let raw = ctx.data().apis.brawlify.get_maps().await?;
    let mut maps = match raw.handler(&ctx, msg).await? {
        Some(maps) => maps,
        None => {
            return ctx
                .components()
                .prompt(
                    msg,
                    CreateEmbed::default().description("No maps were added!"),
                    None,
                )
                .await
        }
    };
    while let Some(map) = maps.pop() {
        let brawl_map = BrawlMap::from(map);
        ctx.data().database.add_map(&brawl_map).await?;
    }
    ctx.components()
        .prompt(
            msg,
            CreateEmbed::default().description("All maps were added!"),
            None,
        )
        .await?;
    Ok(())
}

/// Retrieves all active maps from the game and updates the internal database.
///
/// This command might take a while to run.
#[poise::command(slash_command, rename = "add_maps")]
pub async fn add_map_slash(ctx: BotContext<'_>) -> Result<(), BotError> {
    let reply = {
        let embed = CreateEmbed::default()
            .title("Adding maps to the database")
            .description("This command will add all maps to the database.")
            .footer(CreateEmbedFooter::new("This may take a while."));
        CreateReply::default()
            .ephemeral(true)
            .embed(embed)
            .reply(true)
    };
    let msg = ctx.send(reply).await?;
    add_maps(ctx, &msg).await?;
    Ok(())
}

/// Remove a user from registration with options provided to ban the user's Discord ID or Player
/// Tag.
///
/// This command will automatically disqualify the user from any active tournament they are currently in.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
pub async fn remove_user(ctx: BotContext<'_>, user: User) -> Result<(), BotError> {
    let player = match ctx
        .data()
        .database
        .get_player_by_discord_id(&user.id)
        .await?
    {
        Some(p) => p,
        None => {
            ctx.send(
                CreateReply::default()
                    .content(format!(
                        "User {} has not been registered and can thus not be removed.",
                        user.mention()
                    ))
                    .ephemeral(true),
            )
            .await?;

            return Ok(());
        }
    };
    let buttons = {
        vec![
            CreateButton::new("ban_discord_id_yes")
                .label("Yes")
                .style(ButtonStyle::Success),
            CreateButton::new("ban_discord_id_no")
                .label("No")
                .style(ButtonStyle::Danger),
        ]
    };
    let msg = ctx.send(
        CreateReply::default()
        .content(format!("You are about to remove {}. This will also disqualify them from any tournament they're in.\n\nWould you also like to add this user to the ban list?", user.mention()))
        .components(vec![CreateActionRow::Buttons(buttons)])
        .ephemeral(true)
        ).await?;

    let mut ban_discord_id = false;

    let mut ic = ctx.create_interaction_collector(&msg).await?;
    if let Some(interactions) = &ic.next().await {
        interactions
            .create_response(
                ctx,
                poise::serenity_prelude::CreateInteractionResponse::Acknowledge,
            )
            .await?;
        match interactions.data.custom_id.as_str() {
            "ban_discord_id_yes" => ban_discord_id = true,
            "ban_discord_id_no" => ban_discord_id = false,
            _ => {}
        }
    }

    let buttons = {
        vec![
            CreateButton::new("ban_player_tag_yes")
                .label("Yes")
                .style(ButtonStyle::Success),
            CreateButton::new("ban_player_tag_no")
                .label("No")
                .style(ButtonStyle::Danger),
        ]
    };
    msg.edit(ctx, CreateReply::default()
        .content(format!("Would you like to ban the in-game player tag ({}) associated with this user as well?", player.player_tag))
        .components(vec![CreateActionRow::Buttons(buttons)])
        .ephemeral(true)
        ).await?;

    let mut ban_player_tag = false;
    let mut ic = ctx.create_interaction_collector(&msg).await?;
    if let Some(interactions) = &ic.next().await {
        interactions
            .create_response(
                ctx,
                poise::serenity_prelude::CreateInteractionResponse::Acknowledge,
            )
            .await?;
        match interactions.data.custom_id.as_str() {
            "ban_player_tag_yes" => ban_player_tag = true,
            "ban_player_tag_no" => ban_player_tag = false,
            _ => {}
        }
    }

    let tournaments = ctx
        .data()
        .database
        .get_player_active_tournaments(&ctx.guild_id().unwrap(), &user.id)
        .await?;
    for t in tournaments {
        if t.status == TournamentStatus::Started || t.status == TournamentStatus::Paused {
            disqualify(&ctx, &msg, &t, &user, None).await?;
        } else if t.status == TournamentStatus::Pending {
            ctx.data()
                .database
                .tournament_remove_player(t.tournament_id, &user.id)
                .await?;
        }
    }

    ctx.data().database.delete_user(&user.id).await?;

    let mut success_msg = format!("Successfully removed {}.", user.mention());

    if ban_discord_id {
        ctx.data().database.ban_user(&user.id.to_string()).await?;
        success_msg.push_str(&format!("\n\nBanned Discord ID: {}", user.id.to_string()));
    }

    if ban_player_tag {
        ctx.data().database.ban_user(&player.player_tag).await?;
        success_msg.push_str(&format!("\nBanned Player Tag: {}", player.player_tag));
    }

    msg.edit(
        ctx,
        CreateReply::default()
            .content(success_msg)
            .ephemeral(true)
            .components(vec![]),
    )
    .await?;

    Ok(())
}

/// Unban a user from using the bot.
///
/// This command can be used with either a Discord ID or in-game player tag.
#[poise::command(slash_command, guild_only, check = "is_marshal_or_higher")]
pub async fn unban(ctx: BotContext<'_>, discord_id_or_player_tag: String) -> Result<(), BotError> {
    let discord_id_or_player_tag = match discord_id_or_player_tag.strip_prefix('#') {
        Some(s) => s.to_string(),
        None => discord_id_or_player_tag,
    }
    .to_uppercase();

    if !ctx
        .data()
        .database
        .is_user_banned(&discord_id_or_player_tag)
        .await?
    {
        ctx.send(
            CreateReply::default()
                .content(format!(
                    "User with Discord ID or player tag {} is not currently banned.",
                    discord_id_or_player_tag
                ))
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    ctx.data()
        .database
        .unban_user(&discord_id_or_player_tag)
        .await?;

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Successfully unbanned {}!",
                discord_id_or_player_tag
            ))
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

#[poise::command(
    context_menu_command = "User Profile",
    slash_command,
    guild_only,
    check = "is_marshal_or_higher"
)]
async fn user_profile(
    ctx: BotContext<'_>,
    user: poise::serenity_prelude::User,
) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default()
                .ephemeral(true)
                .embed(CreateEmbed::new().title("Loading")),
        )
        .await?;
    let player = match ctx.get_player_from_discord_id(user.id.to_string()).await {
        Ok(Some(player)) => player,
        Ok(None) => {
            ctx.components().prompt(
                &msg,
                CreateEmbed::new()
                    .title("Profile Not Found")
                    .description("The user has not registered their profile yet. Please run the /menu command to register their profile."),
                None
            ).await?;
            return Ok(());
        }
        Err(e) => {
            ctx.components().prompt(
                &msg,
                CreateEmbed::new().title("Error").description(
                    "An error occurred while fetching the user profile. Please try again later.",
                ),
                None,
            )
            .await?;
            return Err(e);
        }
    };
    display_user_profile_helper(&ctx, &msg, player).await?;
    Ok(())
}

#[poise::command(
    context_menu_command = "Match Menu",
    guild_only,
    check = "is_marshal_or_higher"
)]
async fn view_match_context(
    ctx: BotContext<'_>,
    user: poise::serenity_prelude::User,
) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default()
                .ephemeral(true)
                .embed(CreateEmbed::new().title("Loading...")),
        )
        .await?;
    let tournament = match ctx
        .data()
        .database
        .get_active_tournaments_from_player(&user.id)
        .await?
        .first()
        .cloned()
    {
        Some(t) => t,
        None => {
            let reply = {
                let embed = CreateEmbed::new()
                    .title("Match Context")
                    .description(format!("{} is not in any tournament.", user.mention()))
                    .color(Color::RED);
                CreateReply::default()
                    .reply(true)
                    .ephemeral(true)
                    .embed(embed)
            };
            msg.edit(ctx, reply).await?;
            return Ok(());
        }
    };

    let current_match = match ctx
        .data()
        .database
        .get_current_match(tournament.tournament_id, &user.id)
        .await?
    {
        Some(m) => m,
        None => {
            ctx.components()
                .prompt(
                    &msg,
                    CreateEmbed::new()
                        .title("Match Not Found")
                        .description("This player is not currently in a match."),
                    None,
                )
                .await?;
            return Ok(());
        }
    };
    async fn display_mp(
        ctx: &BotContext<'_>,
        mp: &Option<&MatchPlayer>,
    ) -> Result<String, BotError> {
        match mp {
            Some(p) => {
                let user = p.to_user(ctx).await?;
                Ok(format!("{}", user.mention()))
            }
            None => Ok(
                "No player found. Either this is not yet determined or there is an error."
                    .to_string(),
            ),
        }
    }
    let p1 = display_mp(&ctx, &current_match.match_players.get(0)).await?;
    let p2 = display_mp(&ctx, &current_match.match_players.get(1)).await?;
    let winner = match current_match.get_winning_player() {
        Some(w) => w.to_user(&ctx).await?.mention().to_string(),
        None => "No winner yet".to_string(),
    };

    let embed = CreateEmbed::default()
        .title(format!("Match {}", current_match.match_id))
        .fields(vec![
            ("Match ID", current_match.match_id, true),
            ("Winner", winner, true),
            ("Score", current_match.score, true),
            ("Player 1", p1, false),
            ("Player 2", p2, false),
        ]);

    msg.edit(
        ctx,
        CreateReply::default()
            .content(format!(
                "Here are the details for {}'s match.",
                user.mention()
            ))
            .embed(embed)
            .ephemeral(true),
    )
    .await?;

    Ok(())
}

/// Extracts the conversation between 2 players in a match.
#[poise::command(
    slash_command,
    guild_only,
    check = "is_marshal_or_higher",
    rename = "stalk_mail"
)]
async fn extract_conversation(
    ctx: BotContext<'_>,
    first: User,
    second: User,
) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default()
                .ephemeral(true)
                .embed(CreateEmbed::new().title("Extracting conversation...")),
        )
        .await?;
    return extract_convo_helper(&ctx, &msg, &first, &second).await;
}

const MSG_LIMIT: usize = 2000;

async fn extract_convo_helper(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    first: &User,
    second: &User,
) -> Result<(), BotError> {
    let mails = ctx
        .data()
        .database
        .get_conversation(first.id, second.id)
        .await?;
    let mut content = String::new();
    for mail in mails {
        let timestamp = BattleDateTime::new(mail.id).to_rfc2822();
        let sender = if mail.sender == first.id.to_string() {
            format!("{} {} at {}", first.name, first.id, timestamp)
        } else {
            format!("{} {} at {}", second.name, second.id, timestamp)
        };
        content.push_str(&format!("{}\n", sender));
        content.push_str(&format!("{}\n\n", mail.body));
    }
    let reply = match content.len() {
        0 => {
            let embed = CreateEmbed::new()
                .title("Conversation extraction")
                .description("No conversation found between the 2 players.");
            CreateReply::default()
                .ephemeral(true)
                .embed(embed)
                .reply(true)
        }
        1..=MSG_LIMIT => {
            let embed = CreateEmbed::new()
                .title("Conversation extraction")
                .description(format!(
                    "Conversation between {} and {}\n```{}```",
                    first.mention(),
                    second.mention(),
                    content
                ));
            CreateReply::default()
                .ephemeral(true)
                .embed(embed)
                .reply(true)
        }
        _ => {
            let embed = CreateEmbed::new()
                .title("Conversation extraction")
                .description(format!(
                    "Conversation between {} and {}",
                    first.mention(),
                    second.mention()
                ));
            let attachment = CreateAttachment::bytes(
                content.as_bytes(),
                format!("{}-{}.txt", first.id, second.id),
            );

            CreateReply::default()
                .ephemeral(true)
                .attachment(attachment)
                .components(vec![])
                .embed(embed)
        }
    };
    ctx.send(reply).await?;
    msg.delete(*ctx).await?;

    Ok(())
}
