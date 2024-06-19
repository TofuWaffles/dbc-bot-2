use std::time::{Duration, SystemTime};

use futures::{future::BoxFuture, FutureExt, Stream};
use poise::{
    serenity_prelude::{
        futures::StreamExt, ButtonStyle, ComponentInteraction, ComponentInteractionDataKind,
        CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse, CreateSelectMenu,
        CreateSelectMenuKind, CreateSelectMenuOption,
    },
    CreateReply, ReplyHandle,
};
use prettytable::{row, Table};
use tracing::{error, info, instrument};

use crate::{
    api::{ApiResult, GameApi},
    database::{
        models::{Match, MatchSchedule, PlayerType, Tournament},
        Database,
    },
    reminder::MatchReminderEntry,
    BotData, BotError, Context,
};

use super::CommandsContainer;

/// CommandsContainer for the User commands
pub struct UserCommands;

impl CommandsContainer for UserCommands {
    type Data = BotData;
    type Error = BotError;

    fn get_commands_list() -> Vec<poise::Command<Self::Data, Self::Error>> {
        vec![menu(), register()]
    }
}

/// Amount of time in milliseconds before message interactions (usually buttons) expire for user
/// commands
const USER_CMD_TIMEOUT: u64 = 300000;

/// All-in-one command for all your tournament needs.
#[poise::command(slash_command, prefix_command, guild_only)]
#[instrument]
async fn menu(ctx: Context<'_>) -> Result<(), BotError> {
    info!("User {} has entered the menu", ctx.author().name);

    ctx.defer_ephemeral().await?;

    let user_id = ctx.author().id.to_string();

    let user = ctx.data().database.get_user(&user_id).await?;

    let msg = ctx
        .send(
            CreateReply::default()
                .content("Loading menu...")
                .ephemeral(true),
        )
        .await?;

    let interaction_collector = msg
        .clone()
        .into_message()
        .await?
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(Duration::from_millis(USER_CMD_TIMEOUT))
        .stream();

    match user {
        Some(_) => {
            user_display_menu(ctx, msg, interaction_collector).await?;
        }
        None => {
            // Might make the registration baked into this command later
            ctx.send(
                CreateReply::default()
                    .content("You have not registered your profile yet. Please register first with the /register command.")
                    .ephemeral(true),
            )
            .await?;
        }
    };

    Ok(())
}

/// The user menu entry point.
#[instrument(skip(msg, interaction_collector))]
async fn user_display_menu(
    ctx: Context<'_>,
    msg: ReplyHandle<'_>,
    mut interaction_collector: impl Stream<Item = ComponentInteraction> + Unpin + Send,
) -> Result<(), BotError> {
    info!("User {} has entered the menu home", ctx.author().name);

    let mut player_active_tournaments = ctx
        .data()
        .database
        .get_player_active_tournaments(
            &ctx.guild_id().unwrap().to_string(),
            &ctx.author().id.to_string(),
        )
        .await?;

    if player_active_tournaments.len() < 1 {
        msg.edit(ctx,
            CreateReply::default()
                .content("")
                .embed(
                    CreateEmbed::new()
                        .title("Main Menu")
                        .description("Welcome to the menu! You have not joined a tournament yet. Click on the Tournaments button to join one now!")
                    )
                .components(
                    vec![CreateActionRow::Buttons(
                        vec![
                        CreateButton::new("menu_tournaments")
                        .label("Tournaments")
                        .style(ButtonStyle::Primary)])])
                .ephemeral(true)
                 ).await?;
    } else if player_active_tournaments.len() == 1 {
        msg.edit(
            ctx,
            CreateReply::default()
                .content("")
                .embed(
                    CreateEmbed::new()
                        .title("Main Menu")
                        .description("You're already in a tournament. Good luck!")
                        .fields(vec![
                            (
                                "Tournament Name",
                                player_active_tournaments[0].name.to_owned(),
                                false,
                            ),
                            (
                                "Tournament ID",
                                player_active_tournaments[0].tournament_id.to_string(),
                                false,
                            ),
                            (
                                "Status",
                                format!("{}", player_active_tournaments[0].status),
                                false,
                            ),
                            (
                                "Created At",
                                format!("<t:{}>", player_active_tournaments[0].created_at),
                                false,
                            ),
                        ]),
                )
                .components(vec![CreateActionRow::Buttons(vec![CreateButton::new(
                    "menu_match",
                )
                .label("View Match")
                .style(ButtonStyle::Primary)])])
                .ephemeral(true),
        )
        .await?;
    } else {
        panic!(
            "User {} with ID {} has enetered more than one active tournament",
            ctx.author().name,
            ctx.author().id,
        );
    }

    if let Some(interaction) = &interaction_collector.next().await {
        match interaction.data.custom_id.as_str() {
            "menu_tournaments" => {
                interaction
                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                    .await?;
                msg.edit(
                    ctx,
                    CreateReply::default()
                        .content("Loading tournaments...")
                        .ephemeral(true)
                        .components(vec![]),
                )
                .await?;
                user_display_tournaments(ctx, msg, interaction_collector).await?;
            }
            "menu_match" => {
                interaction
                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                    .await?;
                msg.edit(
                    ctx,
                    CreateReply::default()
                        .content("Loading your match...")
                        .ephemeral(true)
                        .components(vec![]),
                )
                .await?;
                user_display_match(
                    ctx,
                    msg,
                    player_active_tournaments.remove(0),
                    interaction_collector,
                )
                .await?;
            }
            _ => {}
        }
    }

    Ok(())
}

/// The menu that shows the user's match.
#[instrument(skip(msg, interaction_collector))]
async fn user_display_match(
    ctx: Context<'_>,
    msg: ReplyHandle<'_>,
    tournament: Tournament,
    mut interaction_collector: impl Stream<Item = ComponentInteraction> + Unpin + Send,
) -> Result<(), BotError> {
    info!("User {} is viewing their current match", ctx.author().name);

    let bracket = ctx
        .data()
        .database
        .get_match_by_player(&tournament.tournament_id, &ctx.author().id.to_string())
        .await?;

    let match_schedule;

    match bracket {
        Some(ref bracket) => {
            match_schedule = ctx
                .data()
                .database
                .get_match_schedule(&bracket.match_id)
                .await?;
            msg.edit(
                    ctx,
                    CreateReply::default().content("").embed(
                        CreateEmbed::new().title("Match Information.")
                        .description(
                            "Here is all the information for your current match. May the best brawler win!",
                        )
                        .fields(vec![
                            ("Tournament", tournament.name, true),
                            ("Match ID", bracket.match_id.to_owned(), true),
                            ("Round", bracket.round.to_string(), true),
                            ("Player 1",
                             match bracket.player_1_type {
                                PlayerType::Player => format!("<@{}>", bracket.discord_id_1.to_owned().unwrap()),
                                PlayerType::Dummy => "No opponent, please proceed by clicking 'Submit'".to_string(),
                                PlayerType::Pending => "Please wait. Opponent to be determined.".to_string(),
                            },
                             false),
                            ("Player 2", 
                             match bracket.player_2_type {
                                PlayerType::Player => format!("<@{}>", bracket.discord_id_2.to_owned().unwrap()),
                                PlayerType::Dummy => "No opponent for the current match, please proceed by clicking 'Submit'".to_string(),
                                PlayerType::Pending => "Please wait. Opponent to be determined.".to_string(),
                            },
                             false),
                        ]),
                    )
                    .components(
                        vec![
                            CreateActionRow::Buttons(
                                if (bracket.player_1_type == PlayerType::Player) && (bracket.player_2_type == PlayerType::Player) {
                                vec![
                                  CreateButton::new("match_menu_schedule")
                                  .label("Schedule Match")
                                  .style(ButtonStyle::Primary)
                                ,
                    ]} else { vec![] }
                    )]),
                ).await?
        }
        None => {
            msg.edit(
                ctx,
                CreateReply::default()
                    .content("The tournament has not started yet.\n\nNo match currently available.")
                    .ephemeral(true)
                    .components(vec![]),
            )
            .await?;

            return Ok(());
        }
    };

    if let Some(interaction) = &interaction_collector.next().await {
        match interaction.data.custom_id.as_str() {
            "match_menu_schedule" => {
                info!(
                    "User {} is attempting to schedule their match",
                    ctx.author().name
                );
                interaction
                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                    .await?;
                msg.edit(
                    ctx,
                    CreateReply::default()
                        .content("Loading the match scheduling menu...")
                        .ephemeral(true)
                        .components(vec![]),
                )
                .await?;
                match match_schedule {
                    Some(match_schedule) => {
                        user_display_proposed_schedule(
                            ctx,
                            msg,
                            bracket.unwrap(),
                            match_schedule,
                            interaction_collector,
                        )
                        .await?
                    }
                    None => {
                        user_display_match_scheduling(
                            ctx,
                            msg,
                            bracket.unwrap(),
                            interaction_collector,
                        )
                        .await?
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}

#[instrument(skip(msg, interaction_collector))]
fn user_display_proposed_schedule<'a>(
    ctx: Context<'a>,
    msg: ReplyHandle<'a>,
    bracket: Match,
    match_schedule: MatchSchedule,
    mut interaction_collector: impl Stream<Item = ComponentInteraction> + Unpin + Send + 'a,
) -> BoxFuture<'a, Result<(), BotError>> {
    async move {
    let user_id = ctx.author().id.to_string();

    let player_1 = bracket.discord_id_1.clone().unwrap();
    let player_2 = bracket.discord_id_2.clone().unwrap();
    let player_number;

    if user_id == player_1 {
        player_number = 1;
    } else if user_id == player_2 {
        player_number = 2;
    } else {
        error!(
            "Player {} not present in this match!\n\nPlayer 1: {:?}\n\nPlayer 2: {:?}",
            user_id, player_1, player_2
        );
        return Err(format!(
            "Player {} not present in this match!\n\nPlayer 1: {:?}\n\nPlayer 2: {:?}",
            user_id, player_1, player_2
        )
        .into());
    }

    let embed_fields = vec![
        ("Match ID", match_schedule.match_id.clone(), false),
        ("Player 1", format!("<@{}>", player_1), false),
        ("Player 2", format!("<@{}>", player_2), false),
        (
            "Sent at",
            format!("<t:{}>", match_schedule.time_of_proposal.timestamp()),
            false,
        ),
    ];

    if match_schedule.accepted {
        msg.edit(ctx, CreateReply::default()
                 .content("")
                 .embed(CreateEmbed::new()
                        .title("Match schedule accepted")
                        .description(format!("The schedule has been accepted by both players. Be sure to make it on time for your match!\n\nMatch Time: <t:{}>", match_schedule.proposed_time.timestamp()))
                        .fields(embed_fields)
                        )
                 .ephemeral(true)
                 )
            .await?;

        return Ok(());
    }

    if player_number == match_schedule.proposer {
        // User proposed
        msg.edit(ctx, CreateReply::default()
                 .content("")
                 .embed(CreateEmbed::new()
                        .title("You have already scheduled a match with your opponent. They can either accept it or counter with their own schedule.")
                        .description(format!("Match schedule by you for <t:{}>", match_schedule.proposed_time.timestamp()))
                        .fields(embed_fields)
                        )
                 ).await?;
    } else {
        // Opponent proposed
        msg.edit(ctx, CreateReply::default()
                 .content("")
                 .embed(CreateEmbed::new()
                        .title("Your opponent has sent you a schedule. You can either accept the schedule or counter with your own.")
                        .description(format!("Match schedule by you for <t:{}>", match_schedule.proposed_time.timestamp()))
                        .fields(embed_fields)
                        )
                 .components(vec![
                             CreateActionRow::Buttons(vec![
                                                      CreateButton::new("match_schedule_accept").label("Accept").style(ButtonStyle::Success),
                                                      CreateButton::new("match_schedule_reschedule").label("Reschedule").style(ButtonStyle::Primary),
                             ])
                 ])
                 .ephemeral(true)
                 ).await?;
    }

    if let Some(interaction) = &interaction_collector.next().await {
        match interaction.data.custom_id.as_str() {
            "match_schedule_accept" => {
                interaction.create_response(ctx, CreateInteractionResponse::Acknowledge).await?;
                ctx.data()
                    .database
                    .set_match_schedule_accepted(&match_schedule.match_id, true)
                    .await?;

                msg.edit(ctx, CreateReply::default()
                         .content("Refreshing...")
                         .ephemeral(true)
                         .components(vec![])
                         ).await?;

                let new_match_schedule = ctx.data().database.get_match_schedule(&match_schedule.match_id).await?.unwrap();

                user_display_proposed_schedule(ctx, msg, bracket, new_match_schedule, interaction_collector).await?;
            }
            "match_schedule_reschedule" => {
                interaction.create_response(ctx, CreateInteractionResponse::Acknowledge).await?;
                msg.edit(ctx, CreateReply::default().content("Loading match scheduling menu...").components(vec![]).ephemeral(true)).await?;

                user_display_match_scheduling(ctx, msg, bracket, interaction_collector).await?;
            }
            _ => {}
        }
    }

    Ok(())
    }.boxed()
}

/// Menu that allows the user to schedule their match.
#[instrument(skip(msg, interaction_collector))]
fn user_display_match_scheduling<'a>(
    ctx: Context<'a>,
    msg: ReplyHandle<'a>,
    bracket: Match,
    mut interaction_collector: impl Stream<Item = ComponentInteraction> + Unpin + Send + 'a,
) -> BoxFuture<'a, Result<(), BotError>> {
    // Return a BoxFuture to allow recursion
    async move {
        info!(
            "User {} has entered the match scheduling menu",
            ctx.author().name
        );
        let mut hour_options = Vec::new();
        let mut minutes_options = Vec::new();

        for hour in 0..=24 {
            hour_options.push(CreateSelectMenuOption::new(
                hour.to_string(),
                hour.to_string(),
            ));
        }

        for minute in 0..=5 {
            minutes_options.push(CreateSelectMenuOption::new(
                (minute * 10).to_string(),
                (minute * 10).to_string(),
            ));
        }

        msg.edit(ctx,
             CreateReply::default()
             .content("You can propose a match schedule to your opponent here.\nEnter the time you would like to **start** the match from now in hours and minutes.\n\nFor example, if you would like to start your match an hour and 30 minutes from now, you should select 1 for hours and 30 for minutes, then click Submit.\n\nYour opponent can either accept your proposal or counter-offer with their own.\n")
             .components(vec![
                         CreateActionRow::SelectMenu(CreateSelectMenu::new("match_schedule_hour", CreateSelectMenuKind::String { options: hour_options }).placeholder("Hours")),
                         CreateActionRow::SelectMenu(CreateSelectMenu::new("match_schedule_minute", CreateSelectMenuKind::String { options: minutes_options }).placeholder("Minutes")),
                         CreateActionRow::Buttons(vec![CreateButton::new("match_schedule_submit").label("Submit").style(ButtonStyle::Primary)])
             ])
             .ephemeral(true)
        )
        .await?;

        let mut selected_hour = 0;
        let mut selected_minute = 0;

        while let Some(interaction) = &interaction_collector.next().await {
            match interaction.data.custom_id.as_str() {
                "match_schedule_hour" => {
                    interaction
                        .create_response(ctx, CreateInteractionResponse::Acknowledge)
                        .await?;
                    selected_hour = match &interaction.data.kind {
                        ComponentInteractionDataKind::StringSelect { values } => {
                            values[0].parse::<i32>()?
                        }
                        _ => 0,
                    };
                }
                "match_schedule_minute" => {
                    interaction
                        .create_response(ctx, CreateInteractionResponse::Acknowledge)
                        .await?;
                    selected_minute = match &interaction.data.kind {
                        ComponentInteractionDataKind::StringSelect { values } => {
                            values[0].parse::<i32>()?
                        }
                        _ => 0,
                    };
                }
                "match_schedule_submit" => {
                    interaction
                        .create_response(ctx, CreateInteractionResponse::Acknowledge)
                        .await?;
                    msg.edit(
                        ctx,
                        CreateReply::default()
                            .content("Loading match schedule confirmation menu...")
                            .ephemeral(true)
                            .components(vec![]),
                    )
                    .await?;
                    let hours = Duration::from_secs((selected_hour * 60 * 60) as u64);
                    let minutes = Duration::from_secs((selected_minute * 60) as u64);
                    user_display_schedule_confirmation(ctx, msg, bracket, hours, minutes, interaction_collector).await?;
                    break;
                }
                _ => (),
            }
        }

        Ok(())
    }.boxed()
}

/// Menu that allows the user to confirm their selected schedule.
#[instrument(skip(msg, interaction_collector))]
async fn user_display_schedule_confirmation(
    ctx: Context<'_>,
    msg: ReplyHandle<'_>,
    bracket: Match,
    hours: Duration,
    minutes: Duration,
    mut interaction_collector: impl Stream<Item = ComponentInteraction> + Unpin + Send,
) -> Result<(), BotError> {
    info!(
        "User {} has entered the match schedule confirmation menu",
        ctx.author().name
    );
    let now = SystemTime::now();
    let now_unix = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let match_time_unix = (now + hours + minutes)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let user_id = ctx.author().id.to_string();
    let discord_id_1 = bracket.discord_id_1.clone().unwrap_or_default();
    let discord_id_2 = bracket.discord_id_2.clone().unwrap_or_default();

    let notification_channel_id = ctx
        .data()
        .database
        .get_config(&ctx.guild_id().unwrap().to_string())
        .await?
        .unwrap()
        .notification_channel_id;

    let proposer;

    if user_id == discord_id_1 {
        proposer = 1;
    } else if user_id == discord_id_2 {
        proposer = 2;
    } else {
        error!(
            "User id {} is not present in the current match!\n\nID 1: {}\nID 2: {}",
            user_id, discord_id_1, discord_id_2
        );
        return Err(format!(
            "User id {} is not present in the current match!\n\nID 1: {}\nID 2: {}",
            user_id, discord_id_1, discord_id_2
        )
        .into());
    }

    msg.edit(
        ctx,
        CreateReply::default()
            .content("Please check the details of your proposed schedule. Your opponent can either accept or counter with their own schedule proposal.")
            .embed(
                CreateEmbed::new()
                    .title(format!("Match Schedule for Match {}", bracket.match_id))
                    .description(format!("You are proposing to start your match at <t:{}>", match_time_unix))
                    .fields(vec![
                        ("Tournament ID", bracket.tournament_id.to_string(), true),
                        ("Round", bracket.round.to_string(), true),
                        ("Player 1", format!("<@{}>", discord_id_1), false),
                        ("Player 2", format!("<@{}>", discord_id_2), false),
                    ])
            )
            .components(vec![CreateActionRow::Buttons(vec![
                                                 CreateButton::new("match_schedule_confirm").label("Confirm").style(ButtonStyle::Success),
                                                 CreateButton::new("match_schedule_cancel").label("Back").style(ButtonStyle::Danger),
                        ])])
            .ephemeral(true)
            // TODO: Implement a back option.
    )
    .await?;

    if let Some(interaction) = &interaction_collector.next().await {
        match interaction.data.custom_id.as_str() {
            "match_schedule_confirm" => {
                interaction
                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                    .await?;
                ctx.data()
                    .database
                    .create_or_update_match_schedule(
                        &bracket.match_id,
                        match_time_unix,
                        now_unix,
                        proposer,
                    )
                    .await?;
                msg.edit(ctx,
                         CreateReply::default()
                            .content(format!("Congratulations! You've successfully scheduled your next match. Keep an eye out in <#{}> for any updates.", notification_channel_id))
                            .ephemeral(true)
                            .components(vec![])
                    )
                    .await?;
                return Ok(());
            }
            "match_schedule_cancel" => {
                interaction
                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                    .await?;
                msg.edit(
                    ctx,
                    CreateReply::default()
                        .content("Canceling...")
                        .ephemeral(true)
                        .components(vec![]),
                )
                .await?;
                user_display_match_scheduling(ctx, msg, bracket, interaction_collector).await?;
            }
            _ => (),
        }
    }

    Ok(())
}

/// Menu that shows all currently active tournaments to the user for them to join.
#[instrument(skip(msg, interaction_collector))]
async fn user_display_tournaments(
    ctx: Context<'_>,
    msg: ReplyHandle<'_>,
    mut interaction_collector: impl Stream<Item = ComponentInteraction> + Unpin + Send,
) -> Result<(), BotError> {
    info!(
        "User {} has entered the tournaments menu",
        ctx.author().name
    );
    let guild_id = ctx.guild_id().unwrap().to_string();
    let tournaments = ctx
        .data()
        .database
        .get_active_tournaments(&guild_id)
        .await?;

    if tournaments.len() < 1 {
        let config = ctx.data().database.get_config(&guild_id).await?.unwrap();
        msg.edit(ctx, CreateReply::default()
                 .content(format!("There are no tournaments currently available. Keep an eye out in <#{}> for new and upcoming tournaments!", config.announcement_channel_id))
                 .components(vec![])
                 .ephemeral(true)
            )
            .await?;
        return Ok(());
    }

    let mut table = Table::new();
    table.set_titles(row!["No.", "Name", "Status"]);

    let mut interaction_ids = Vec::new();

    let mut tournament_buttons = Vec::new();

    for (i, tournament) in tournaments.iter().enumerate() {
        // Add 1 to the loop iteration so that the user-facing tournament numbers start at 1
        // instead of 0
        table.add_row(row![
            i + 1,
            &tournament.name,
            &tournament.status.to_string()
        ]);

        interaction_ids.push(format!("join_tournament_{}", tournament.tournament_id));

        tournament_buttons.push(
            CreateButton::new(interaction_ids.last().unwrap())
                .label((i + 1).to_string())
                .style(ButtonStyle::Primary),
        );
    }

    msg.edit(
        ctx,
        CreateReply::default()
            .content(format!(
                "Here are all the active tournaments in this server.\n\nTo join a tournament, click the button with the number corresponding to the one you wish to join.\n```\n{}\n```",
                table.to_string()
            ))
            .components(vec![CreateActionRow::Buttons(tournament_buttons)]),
    )
    .await?;

    while let Some(interaction) = &interaction_collector.next().await {
        match interaction_ids
            .iter()
            .position(|id| id == interaction.data.custom_id.as_str())
        {
            Some(tournament_number) => {
                interaction
                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                    .await?;
                ctx.data()
                    .database
                    .enter_tournament(
                        &tournaments[tournament_number].tournament_id,
                        &ctx.author().id.to_string(),
                    )
                    .await?;
                msg.edit(
                    ctx,
                    CreateReply::default()
                        .content("Congratulations! You have successfully entered the tournament.\n\nSee you on the battle field!")
                        .ephemeral(true)
                        .components(vec![]),
                ).await?;

                return Ok(());
            }
            None => (),
        };
    }

    Ok(())
}

/// Register your in-game profile with the bot.
#[poise::command(slash_command, prefix_command, guild_only)]
async fn register(ctx: Context<'_>, player_tag: String) -> Result<(), BotError> {
    let user_id = ctx.author().id.to_string();

    if ctx.data().database.get_user(&user_id).await?.is_some() {
        ctx.send(
            CreateReply::default()
                .content("You have already registered your profile.")
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    let api_result = ctx.data().game_api.get_player(&player_tag).await?;
    match api_result {
        ApiResult::Ok(player) => {
            let msg = ctx
                .send(
                    CreateReply::default()
                        .embed(
                            CreateEmbed::new()
                                .title(format!("**{} ({})**", player.name, player.tag))
                                .description("**Please confirm that this is your profile**")
                                .thumbnail(format!(
                                    "https://cdn-old.brawlify.com/profile/{}.png",
                                    player.icon.get("id").unwrap_or(&1)
                                ))
                                .fields(vec![
                                    ("Trophies", player.trophies.to_string(), true),
                                    (
                                        "Highest Trophies",
                                        player.highest_trophies.to_string(),
                                        true,
                                    ),
                                    (
                                        "3v3 Victories",
                                        player.three_vs_three_victories.to_string(),
                                        true,
                                    ),
                                    ("Solo Victories", player.solo_victories.to_string(), true),
                                    ("Duo Victories", player.duo_victories.to_string(), true),
                                    ("Club", player.club.unwrap_or_default().name, true),
                                ])
                                .timestamp(ctx.created_at())
                                .color(0x0000FF),
                        )
                        .components(vec![CreateActionRow::Buttons(vec![
                            CreateButton::new("confirm_register")
                                .label("Confirm")
                                .style(ButtonStyle::Primary),
                            CreateButton::new("cancel_register")
                                .label("Cancel")
                                .style(ButtonStyle::Danger),
                        ])])
                        .ephemeral(true),
                )
                .await?;

            let mut interaction_collector = msg
                .clone()
                .into_message()
                .await?
                .await_component_interaction(&ctx.serenity_context().shard)
                .timeout(Duration::from_millis(USER_CMD_TIMEOUT))
                .stream();

            if let Some(interaction) = &interaction_collector.next().await {
                match interaction.data.custom_id.as_str() {
                    "confirm_register" => {
                        interaction
                            .create_response(ctx, CreateInteractionResponse::Acknowledge)
                            .await?;
                        ctx.data()
                            .database
                            .create_user(&user_id, &player.tag)
                            .await?;
                        msg.edit(
                            ctx,
                            CreateReply::default()
                                .content("You have successfully registered your profile.")
                                .ephemeral(true)
                                .components(vec![]),
                        )
                        .await?;
                    }
                    "cancel_register" => {
                        interaction
                            .create_response(ctx, CreateInteractionResponse::Acknowledge)
                            .await?;
                        msg.edit(
                            ctx,
                            CreateReply::default()
                                .content("Canceled profile registration")
                                .ephemeral(true)
                                .components(vec![]),
                        )
                        .await?;
                    }
                    _ => {}
                }
            }
        }
        ApiResult::NotFound => {
            ctx.send(
                CreateReply::default()
                    .content("A profile with that tag was not found. Please ensure that you have entered the correct tag.")
                    .ephemeral(true),
            )
            .await?;
        }
        ApiResult::Maintenance => {
            ctx.send(
                CreateReply::default()
                    .content("The Brawl Stars API is currently undergoing maintenance. Please try again later.")
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}
