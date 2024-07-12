use std::{str::FromStr, time::Duration};

use anyhow::anyhow;
use futures::Stream;
use poise::{
    serenity_prelude::{
        futures::StreamExt, ButtonStyle, ChannelId, ComponentInteraction, CreateActionRow,
        CreateButton, CreateEmbed, CreateInteractionResponse, CreateMessage,
    },
    CreateReply, Modal, ReplyHandle,
};
use prettytable::{row, Table};
use serde_json::json;
use tracing::{info, instrument};

use crate::{
    api::{ApiResult, GameApi},
    commands::checks::{is_config_set, is_tournament_paused},
    database::{
        models::{
            PlayerNumber::{Player1, Player2},
            PlayerType, Tournament, TournamentStatus, User,
        },
        Database,
    },
    BotContext, BotData, BotError,
};

use super::CommandsContainer;

/// CommandsContainer for the User commands
pub struct UserCommands;

impl CommandsContainer for UserCommands {
    type Data = BotData;
    type Error = BotError;

    fn get_all() -> Vec<poise::Command<Self::Data, Self::Error>> {
        vec![menu()]
    }
}

/// Amount of time in milliseconds before message interactions (usually buttons) expire for user
/// commands
const USER_CMD_TIMEOUT: u64 = 120000;

/// All-in-one command for all your tournament needs.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    check = "is_config_set",
    check = "is_tournament_paused"
)]
#[instrument]
async fn menu(ctx: BotContext<'_>) -> Result<(), BotError> {
    info!("User {} has entered the menu", ctx.author().name);

    ctx.defer_ephemeral().await?;

    let user_id = ctx.author().id.to_string();

    let user = ctx
        .data()
        .database
        .get_player_by_discord_id(&user_id)
        .await?;

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
            msg.edit(
                ctx,
                CreateReply::default()
                    .content("Loading registration page...")
                    .components(vec![])
                    .ephemeral(true),
            )
            .await?;
            user_display_registration(ctx, msg, interaction_collector).await?;
        }
    };

    Ok(())
}

/// Display the main menu to the registered user.
#[instrument(skip(msg, interaction_collector))]
async fn user_display_menu(
    ctx: BotContext<'_>,
    msg: ReplyHandle<'_>,
    mut interaction_collector: impl Stream<Item = ComponentInteraction> + Unpin,
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

    if player_active_tournaments.is_empty() {
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

/// Display match information to the user.
#[instrument(skip(msg, interaction_collector))]
async fn user_display_match(
    ctx: BotContext<'_>,
    msg: ReplyHandle<'_>,
    tournament: Tournament,
    mut interaction_collector: impl Stream<Item = ComponentInteraction> + Unpin,
) -> Result<(), BotError> {
    info!("User {} is viewing their current match", ctx.author().name);

    let bracket_opt = ctx
        .data()
        .database
        .get_match_by_player(tournament.tournament_id, &ctx.author().id.to_string())
        .await?;

    let bracket = match bracket_opt {
        Some(ref bracket) => {
            let reply;
            if bracket.player_1_type == PlayerType::Dummy
                || bracket.player_2_type == PlayerType::Dummy
            {
                // Automatically advance the player to the next round if the opponent is a dummy
                // (a bye round)
                ctx.data()
                    .database
                    .set_winner(
                        &bracket.match_id,
                        bracket
                            .get_player_number(&ctx.author().id.to_string())
                            .ok_or(anyhow!(
                                "Player <@{}> is not in this match {}",
                                ctx.author().id.to_string(),
                                bracket.match_id
                            ))?,
                    )
                    .await?;
                reply = CreateReply::default().content("").embed(
                        CreateEmbed::new().title("Match Information.")
                        .description(
                            "You have no opponents for the current round. See you in the next round, partner!",
                        )
                        .fields(vec![
                            ("Tournament", tournament.name, true),
                            ("Match ID", bracket.match_id.to_owned(), true),
                            ("Round", bracket.round.to_string(), true),
                        ]),
                    );
            } else if bracket.player_1_type == PlayerType::Pending
                || bracket.player_2_type == PlayerType::Pending
            {
                // Pending is not currently in use, but we check for it anyway
                reply = CreateReply::default().content("").embed(
                    CreateEmbed::new()
                        .title("Match Information.")
                        .description("Your opponent has yet to be determined. Please be patient.")
                        .fields(vec![
                            ("Tournament", tournament.name, true),
                            ("Match ID", bracket.match_id.to_owned(), true),
                            ("Round", bracket.round.to_string(), true),
                        ]),
                );
            } else {
                let player_number = bracket
                    .get_player_number(&ctx.author().id.to_string())
                    .ok_or(anyhow!(
                        "Player <@{}> is not in match {}",
                        ctx.author().id.to_string(),
                        bracket.match_id
                    ))?;
                // We don't want to show the player the ready button if they're already ready
                let button_components = match player_number {
                    Player1 => {
                        if !bracket.player_1_ready {
                            vec![CreateActionRow::Buttons(vec![CreateButton::new(
                                "match_menu_ready",
                            )
                            .label("Ready")
                            .style(ButtonStyle::Success)])]
                        } else {
                            vec![]
                        }
                    }
                    Player2 => {
                        if !bracket.player_2_ready {
                            vec![CreateActionRow::Buttons(vec![CreateButton::new(
                                "match_menu_ready",
                            )
                            .label("Ready")
                            .style(ButtonStyle::Success)])]
                        } else {
                            vec![]
                        }
                    }
                };
                reply = CreateReply::default().content("").embed(
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
                                PlayerType::Player => format!("<@{}>", bracket.discord_id_1.to_owned().ok_or(anyhow!("Player 1 is set to type Player but has no Discord ID in match {}", bracket.match_id))?),
                                PlayerType::Dummy => "No opponent, please proceed by clicking 'Submit'".to_string(),
                                PlayerType::Pending => "Please wait. Opponent to be determined.".to_string(),
                            },
                             false),
                            ("Player 2", 
                             match bracket.player_2_type {
                                PlayerType::Player => format!("<@{}>", bracket.discord_id_2.to_owned().ok_or(anyhow!("Player 2 is set to type Player but has not Discord ID in match {}", bracket.match_id))?),
                                PlayerType::Dummy => "No opponent for the current match, please proceed by clicking 'Submit'".to_string(),
                                PlayerType::Pending => "Please wait. Opponent to be determined.".to_string(),
                            },
                             false),
                        ]),
                    )
                    .components(button_components);
            }

            msg.edit(ctx, reply).await?;

            bracket
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
        if interaction.data.custom_id.as_str() == "match_menu_ready" {
            interaction
                .create_response(ctx, CreateInteractionResponse::Acknowledge)
                .await?;

            msg.edit(
                    ctx,
                    CreateReply::default()
                    .content("Your have set yourself to ready. A notification has been sent to your opponent to let them know.\n\nBe sure to play your matches and hit the \"Submit\" button when you're done.")
                    .components(vec![])
                    .ephemeral(true))
                    .await?;

            let player_number = bracket
                .get_player_number(&ctx.author().id.to_string())
                .unwrap();

            let player_1_id = bracket.discord_id_1.clone().ok_or(anyhow!(
                "Player 1 type is set to Player but has not Discord ID in match {}",
                bracket.match_id
            ))?;
            let player_2_id = bracket.discord_id_2.clone().ok_or(anyhow!(
                "Player 2 type is set to Player but has not Discord ID in match {}",
                bracket.match_id
            ))?;

            ctx.data()
                .database
                .set_ready(&bracket.match_id, &player_number)
                .await?;

            let notification_message = match player_number {
                Player1 => {
                    if bracket.player_2_ready {
                        format!("<@{}> and <@{}>.\n\nBoth players have readied up. Please complete your matches and press the \"Submit\" button when you have done so. Best of luck!", player_1_id, player_2_id)
                    } else {
                        format!("<@{}>.\n\nYour opponent <@{}> has readied up. You are advised to ready up using the /menu command or get your match in by clicking \"Submit\" in the menu. Failure to do so may result in automatic disqualification.", player_2_id, player_1_id)
                    }
                }
                Player2 => {
                    if bracket.player_1_ready {
                        format!("<@{}> and <@{}>.\n\nBoth players have readied up. Please complete your matches and press the \"Submit\" button when you have done so. Best of luck!", player_1_id, player_2_id)
                    } else {
                        format!("<@{}>.\n\nYour opponent <@{}> has readied up. You are advised to ready up using the /menu command or get your match in by clicking \"Submit\" in the menu. Failure to do so may result in automatic disqualification.", player_1_id, player_2_id)
                    }
                }
            };

            let notification_channel = ChannelId::from_str(
                &ctx.data()
                    .database
                    .get_config(&ctx.guild_id().unwrap().to_string())
                    .await?
                    .unwrap()
                    .notification_channel_id,
            )?;

            notification_channel
                .send_message(ctx, CreateMessage::default().content(notification_message))
                .await?;
        }
    }
    Ok(())
}

/// Display all active (and not started) tournaments to the user who has not yet joined a
/// tournament.
#[instrument(skip(msg, interaction_collector))]
async fn user_display_tournaments(
    ctx: BotContext<'_>,
    msg: ReplyHandle<'_>,
    mut interaction_collector: impl Stream<Item = ComponentInteraction> + Unpin,
) -> Result<(), BotError> {
    info!(
        "User {} has entered the tournaments menu",
        ctx.author().name
    );
    let guild_id = ctx.guild_id().unwrap().to_string();
    let tournaments: Vec<Tournament> = ctx
        .data()
        .database
        .get_active_tournaments(&guild_id)
        .await?
        .into_iter()
        .filter(|tournament| tournament.status == TournamentStatus::Pending)
        .collect();

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

    if !tournament_buttons.is_empty() {
        msg.edit(
        ctx,
        CreateReply::default()
            .content(format!(
                "Here are all the active tournaments in this server.\n\nTo join a tournament, click the button with the number corresponding to the one you wish to join.\n```\n{}\n```",
                table
            ))
            .components(vec![CreateActionRow::Buttons(tournament_buttons)]),
    )
    .await?;
    } else {
        let announcement_channel_id = ctx
            .data()
            .database
            .get_config(&guild_id)
            .await?
            .unwrap()
            .announcement_channel_id;
        msg.edit(
        ctx,
        CreateReply::default()
            .content(format!(
                "There are no tournaments currently available. Be sure to check out <#{}> for any new tournaments on the horizon!",
                announcement_channel_id
            ))
            .components(vec![]),
    )
    .await?;

        return Ok(());
    }

    while let Some(interaction) = &interaction_collector.next().await {
        match interaction_ids.iter().position(|id| id == interaction.data.custom_id.as_str()) {
            Some(tournament_number) => {
                interaction.create_response(ctx, CreateInteractionResponse::Acknowledge).await?;
                ctx.data().database.enter_tournament(tournaments[tournament_number].tournament_id, &ctx.author().id.to_string()).await?;
                msg.edit(
                    ctx,
                    CreateReply::default()
                        .content("Congratulations! You have successfully entered the tournament.\n\nSee you on the battle field!")
                        .ephemeral(true)
                        .components(vec![]),
                )
            }.await?,
            None => continue,
        };
    }

    Ok(())
}

/// Registers the user's in-game profile with the bot.
#[instrument(skip(msg, interaction_collector))]
async fn user_display_registration(
    ctx: BotContext<'_>,
    msg: ReplyHandle<'_>,
    mut interaction_collector: impl Stream<Item = ComponentInteraction> + Unpin,
) -> Result<(), BotError> {
    let mut user = User::default();
    msg.edit(ctx,
             CreateReply::default()
             .content("You'll need to register your in-game account with us to enter one of our tournaments.\n\nClick the button below to get started.")
             .components(vec![
                         CreateActionRow::Buttons(vec![
                                                  CreateButton::new("player_profile_registration")
                                                       .label("Register")
                                                       .style(ButtonStyle::Primary)])])
             .ephemeral(true))
        .await?;

    #[derive(Debug, Modal)]
    #[name = "Profile Registration"]
    struct ProfileRegistrationModal {
        #[name = "Player Tag"]
        #[placeholder = "Your in-game player tag (without #)"]
        #[min_length = 4]
        #[max_length = 10]
        player_tag: String,
    }

    let mut player_tag: String = Default::default();

    if let Some(interaction) = interaction_collector.next().await {
        match interaction.data.custom_id.as_str() {
            "player_profile_registration" => {
                player_tag = poise::execute_modal_on_component_interaction::<
                    ProfileRegistrationModal,
                >(ctx, interaction, None, None)
                .await?
                .ok_or(anyhow!("Modal interaction from <@{}> returned None. This may mean that the modal has timed out.", ctx.author().id.to_string()))?
                .player_tag
                .to_uppercase();

                if player_tag.starts_with('#') {
                    player_tag.remove(0);
                }
                user.player_tag = player_tag;
            }
            _ => {
                return Err(anyhow!(
                    "Unknown interaction from player registration.\n\nUser: {}",
                    ctx.author()
                ))
            }
        }
    }

    let user_id = ctx.author().id.to_string();
    if ctx
        .data()
        .database
        .get_player_by_player_tag(&user.player_tag)
        .await?
        .is_some()
    {
        msg.edit(ctx, CreateReply::default().content("This game account is currently registered with another user. Please register with another game account.").components(vec![]).ephemeral(true)).await?;

        return Ok(());
    }

    msg.edit(
        ctx,
        CreateReply::default()
            .content("Getting your game account details, please wait...")
            .components(vec![])
            .ephemeral(true),
    )
    .await?;

    let api_result = ctx.data().game_api.get_player(&user.player_tag).await?;
    match api_result {
        ApiResult::Ok(player) => {
            msg.edit(
                ctx,
                CreateReply::default()
                    .embed(
                        CreateEmbed::new()
                            .title(format!("**{} ({})**", player.name, player.tag))
                            .description("**Please confirm that this is your profile**")
                            .thumbnail(format!(
                                "https://cdn-old.brawlify.com/profile/{}.png",
                                player.icon.id
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

            if let Some(interaction) = &interaction_collector.next().await {
                match interaction.data.custom_id.as_str() {
                    "confirm_register" => {
                        user.brawlers = json!(player.brawlers);
                        user.player_name = player.name;
                        user.icon = player.icon.id;
                        user.trophies = player.trophies as i32;
                        user.discord_name = ctx.author().name.clone();
                        user.discord_id = user_id.clone();
                        interaction
                            .create_response(ctx, CreateInteractionResponse::Acknowledge)
                            .await?;
                        ctx.data().database.create_user(&user).await?;
                        msg.edit(
                            ctx,
                            CreateReply::default()
                                .content("You have successfully registered your profile. Go ahead and check out /menu again!")
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
