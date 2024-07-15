use anyhow::anyhow;
use futures::Stream;
use poise::{
    serenity_prelude::{
        futures::StreamExt, ButtonStyle, ChannelId, Color, ComponentInteraction, CreateActionRow,
        CreateButton, CreateEmbed, CreateMessage,
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
    log::{self, Log},
    utils::{
        discord::{modal, select_options},
        shorthand::BotContextExt,
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
    // info!("User {} has entered the menu", ctx.author().name);
    ctx.defer_ephemeral().await?;
    let user = ctx
        .get_player_from_discord_id(ctx.author().to_owned())
        .await?;
    let embed = CreateEmbed::new()
        .title("Menu")
        .description("Please wait while we load the menu.")
        .color(0x0000FF);
    let msg = ctx
        .send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    let interaction_collector = ctx.create_interaction_collector(&msg).await?;

    return match user {
        Some(_) => user_display_menu(&ctx, &msg).await,
        None => {
            ctx.prompt(
                &msg,
                CreateEmbed::new()
                    .title("Registration Page Menu")
                    .description("Loading registration page...")
                    .color(Color::BLUE),
                None,
            )
            .await?;
            user_display_registration(&ctx, &msg, interaction_collector).await
        }
    };
}

/// Display the main menu to the registered user.
#[instrument(skip(msg))]
async fn user_display_menu(ctx: &BotContext<'_>, msg: &ReplyHandle<'_>) -> Result<(), BotError> {
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
        let buttons = vec![
            CreateButton::new("menu_tournaments")
                .label("Tournaments")
                .style(ButtonStyle::Primary),
            CreateButton::new("profile")
                .label("Profile")
                .style(ButtonStyle::Primary),
            CreateButton::new("deregister")
                .label("Deregister")
                .style(ButtonStyle::Danger),
        ];
        ctx.prompt(
            msg,
            CreateEmbed::new().title("Main Menu").description("Welcome to the menu! You have not joined a tournament yet. Click on the Tournaments button to join one now!").color(Color::BLUE),
            buttons
        ).await?;
    } else if player_active_tournaments.len() == 1 {
        let embed = CreateEmbed::new()
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
            ]);
        let buttons = vec![
            CreateButton::new("menu_match")
                .label("View Match")
                .style(ButtonStyle::Primary),
            CreateButton::new("leave_tournament")
                .label("Leave Tournament")
                .style(ButtonStyle::Danger),
            CreateButton::new("profile")
                .label("Profile")
                .style(ButtonStyle::Primary),
        ];
        ctx.prompt(msg, embed, buttons).await?;
    } else {
        return Err(anyhow!(
            "User {} with ID {} has enetered more than one active tournament",
            ctx.author().name,
            ctx.author().id,
        ));
    }
    let mut ic = ctx.create_interaction_collector(msg).await?;
    while let Some(interaction) = &ic.next().await {
        match interaction.data.custom_id.as_str() {
            "menu_tournaments" => {
                interaction.defer(ctx.http()).await?;
                ctx.prompt(
                    msg,
                    CreateEmbed::new()
                        .title("Tournaments")
                        .description("Loading tournaments...")
                        .color(Color::BLUE),
                    None,
                )
                .await?;
                return user_display_tournaments(ctx, msg).await;
            }
            "deregister" => {
                interaction.defer(ctx.http()).await?;
                return deregister(ctx, msg).await;
            }
            "profile" => {
                interaction.defer(ctx.http()).await?;
                return display_user_profile(ctx, msg).await;
            }
            "menu_match" => {
                interaction.defer(ctx.http()).await?;
                ctx.prompt(
                    msg,
                    CreateEmbed::new()
                        .title("Match Information")
                        .description("Loading your match...")
                        .color(Color::BLUE),
                    None,
                )
                .await?;

                return user_display_match(ctx, msg, player_active_tournaments.remove(0)).await;
            }
            "leave_tournament" => {
                interaction.defer(ctx.http()).await?;
                leave_tournament(ctx, msg).await?;
            }
            _ => {
                continue;
            }
        }
    }

    Ok(())
}

/// Display match information to the user.
#[instrument(skip(msg))]
async fn user_display_match(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    tournament: Tournament,
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

            msg.edit(*ctx, reply).await?;
            bracket
        }
        None => {
            return ctx.prompt(
                msg,
                CreateEmbed::new().title("Match Information").description("You are not currently in a match. Please wait for the next round to begin.").color(Color::RED), 
                None
            ).await;
        }
    };
    let mut ic = ctx.create_interaction_collector(msg).await?;
    while let Some(interaction) = &ic.next().await {
        if interaction.data.custom_id.as_str() == "match_menu_ready" {
            interaction.defer(ctx.http()).await?;
            ctx.prompt(
                msg,
                CreateEmbed::new()
                    .title("Ready Confirmation")
                    .description("You have set yourself to ready. A notification has been sent to your opponent to let them know.\n\nBe sure to play your matches and hit the \"Submit\" button when you're done.")
                    .color(Color::DARK_GREEN),
                None,
            ).await?;

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
                        format!(
                            r#"<@{}> and <@{}>.\n\nBoth players have readied up. Please complete your matches and press the "Submit" button when you have done so. Best of luck!"#,
                            player_1_id, player_2_id
                        )
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

            let notification_channel = ChannelId::new(tournament.notification_channel_id.parse()?);
            notification_channel
                .send_message(ctx, CreateMessage::default().content(notification_message))
                .await?;
        } else {
            continue;
        }
    }
    Ok(())
}

/// Display all active (and not started) tournaments to the user who has not yet joined a
/// tournament.
#[instrument(skip(msg))]
async fn user_display_tournaments(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
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
    for (i, tournament) in tournaments.iter().enumerate() {
        // Add 1 to the loop iteration so that the user-facing tournament numbers start at 1
        // instead of 0
        table.add_row(row![
            i + 1,
            &tournament.name,
            &tournament.status.to_string()
        ]);
    }

    let selected_tournament = if !tournaments.is_empty() {
        select_options(
            ctx,
            msg,
            "Tournament Enrollment",
            "Here are all the active tournaments in this server.\n\nTo join a tournament, click the button with the number corresponding to the one you wish to join.",
            &tournaments
        ).await?
    } else {
        let announcement_channel_id = ctx
            .data()
            .database
            .get_config(&guild_id)
            .await?
            .unwrap()
            .announcement_channel_id;
        ctx.prompt(
            msg,
            CreateEmbed::new()
                .title("Tournament Enrollment")
                .description(format!("There are no tournaments currently available. Be sure to check out <#{}> for any new tournaments on the horizon!", announcement_channel_id))
                .color(Color::RED),
           None
        ).await?;
        return Ok(());
    };
    match ctx
        .data()
        .database
        .enter_tournament(
            selected_tournament.parse::<i32>()?,
            &ctx.author().id.to_string(),
        )
        .await
    {
        Ok(_) => {
            ctx.log(
                "Tournament enrollment success",
                format!(
                    "User {} has joined tournament {}",
                    ctx.author().name,
                    selected_tournament
                ),
                log::State::SUCCESS,
                log::Model::TOURNAMENT,
            )
            .await?;
            ctx.prompt(
                msg,
                CreateEmbed::new()
                    .title("Tournament Enrollment")
                    .description("You have successfully joined the tournament! Good luck!")
                    .color(Color::DARK_GREEN),
                None,
            )
            .await?;
        }
        Err(e) => {
            ctx.log(
                "Tournament enrollment failure",
                format!(
                    "User {} failed to join tournament {}\n Error detail: {}",
                    ctx.author().name,
                    selected_tournament,
                    e
                ),
                log::State::FAILURE,
                log::Model::TOURNAMENT,
            )
            .await?;
            ctx.prompt(
                msg,
                CreateEmbed::new()
                    .title("Tournament Enrollment")
                    .description("You have already joined this tournament. Please wait for the tournament to start.")
                    .color(Color::RED),
                None,
            )
            .await?;
        }
    }
    Ok(())
}

/// Registers the user's in-game profile with the bot.
#[instrument(skip(msg, interaction_collector))]
async fn user_display_registration(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    mut interaction_collector: impl Stream<Item = ComponentInteraction> + Unpin,
) -> Result<(), BotError> {
    let mut user = User::default();
    let buttons = vec![CreateButton::new("player_profile_registration")
        .label("Register")
        .style(ButtonStyle::Primary)];
    ctx.prompt(
        msg,
        CreateEmbed::new()
            .title("Registration Page")
            .description("Welcome to the registration page! Please click the button below to register your in-game profile.")
            .color(Color::BLUE),
        buttons
    ).await?;
    #[derive(Debug, Modal)]
    #[name = "Profile Registration"]
    struct ProfileRegistrationModal {
        #[name = "Player Tag"]
        #[placeholder = "Your in-game player tag (without #)"]
        #[min_length = 4]
        #[max_length = 10]
        player_tag: String,
    }

    if let Some(interaction) = interaction_collector.next().await {
        interaction.defer(ctx.http()).await?;
        match interaction.data.custom_id.as_str() {
            "player_profile_registration" => {
                let embed = CreateEmbed::new()
                .title("Profile Registration")
                .description("Please enter your in-game player tag (without the #) The tutorial below would help you find your player tag (wait patiently for the gif to load)")
                .image("https://i.imgur.com/bejTDlO.gif")
                .color(0x0000FF);
                let mut player_tag = modal::<ProfileRegistrationModal>(ctx, msg, embed)
                    .await?
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
    if ctx.get_player_from_tag(&user.player_tag).await?.is_some() {
        ctx.prompt(
        msg,
        CreateEmbed::new()
            .title("Registration Error")
            .description("This game account is currently registered with another user. Please register with another game account.")
            .color(Color::RED),
      None).await?;
        ctx.log(
            "Attempted registration failure",
            format!("{} is attempted to be registered!", user.player_tag),
            crate::log::State::FAILURE,
            crate::log::Model::PLAYER,
        )
        .await?;
        return Ok(());
    }

    ctx.prompt(
        msg,
        CreateEmbed::new()
            .title("Profile Registration")
            .description("Please wait while we fetch your game account details.")
            .color(Color::BLUE),
        None,
    )
    .await?;
    let api_result = ctx.data().game_api.get_player(&user.player_tag).await?;
    match api_result {
        ApiResult::Ok(player) => {
            let embed = {
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
                    .color(0x0000FF)
            };
            match ctx.confirmation(msg, embed).await? {
                true => {
                    user.brawlers = json!(player.brawlers);
                    user.player_name = player.name.clone();
                    user.icon = player.icon.id;
                    user.trophies = player.trophies;
                    user.discord_name = ctx.author().name.clone();
                    user.discord_id = user_id.clone();
                    ctx.data().database.create_user(&user).await?;
                    ctx.prompt(msg,
                            CreateEmbed::new()
                                .title("Registration Success!")
                                .description("Your profile has been successfully registered! Please run this command again to access Player menu!"),
                            None).await?;
                    ctx.log(
                        "Registration success!",
                        format!("Tag {} registered!", user.player_tag),
                        crate::log::State::SUCCESS,
                        crate::log::Model::PLAYER,
                    )
                    .await?;
                }
                false => {
                    ctx.prompt(
                        msg,
                        CreateEmbed::new()
                            .title("Registration Cancelled")
                            .description("You have cancelled the registration process. Please run this command again to register your profile.")
                            .color(Color::RED),
                        None
                    ).await?;
                }
            }
        }
        ApiResult::NotFound => {
            ctx.prompt(
                msg,
                CreateEmbed::new()
                    .title("Player Not Found")
                    .description("The player tag you entered was not found. Please try again."),
                None,
            )
            .await?;
            ctx.log(
                "Player",
                format!("Player tag {} not found", user.player_tag),
                crate::log::State::FAILURE,
                crate::log::Model::PLAYER,
            )
            .await?;
        }
        ApiResult::Maintenance => {
            ctx.prompt(
                msg,
                CreateEmbed::new()
                    .title("Maintenance")
                    .description("The Brawl Stars API is currently undergoing maintenance. Please try again later."),
               None,
            )
            .await?;
            ctx.log(
                "API",
                "Brawl Stars API is currently undergoing maintenance",
                crate::log::State::FAILURE,
                crate::log::Model::API,
            )
            .await?;
        }
    }
    Ok(())
}

async fn display_user_profile(ctx: &BotContext<'_>, msg: &ReplyHandle<'_>) -> Result<(), BotError> {
    let user = match ctx
        .get_player_from_discord_id(ctx.author().to_owned())
        .await?
    {
        Some(player) => ctx.data().database.get_user_by_player(player).await?,
        None => {
            ctx.prompt(
                msg,
                CreateEmbed::new()
                    .title("Profile Not Found")
                    .description("You have not registered your profile yet. Please run the /menu command to register your profile."), None).await?;
            ctx.log(
                "Player not found in the database!",
                "User who runs this command does not own any profile!",
                log::State::FAILURE,
                log::Model::PLAYER,
            )
            .await?;
            return Ok(());
        }
    };
    let user = match user {
        None => {
            ctx.prompt(
                msg,
                CreateEmbed::new()
                    .title("Profile Not Found")
                    .description("You have not registered your profile yet. Please run the /menu command to register your profile."),
                None
                ).await?;
            ctx.log(
                "Player not found in the database!",
                "User who runs this command does not own any profile!",
                log::State::FAILURE,
                log::Model::PLAYER,
            )
            .await?;
            return Ok(());
        }
        Some(user) => user,
    };
    let embed = {
        CreateEmbed::new()
            .title(format!("**{} ({})**", user.player_name, user.player_tag))
            .description("**Here is your profile information that we collected**")
            .thumbnail(format!(
                "https://cdn-old.brawlify.com/profile/{}.png",
                user.icon
            ))
            .fields(vec![
                ("Discord Id", &user.discord_id, true),
                ("Discord name", &user.discord_name, true),
                ("Player tag", &format!("#{}", user.player_tag), true),
                ("Trophies", &user.trophies.to_string(), true),
                (
                    "Brawler count:",
                    &user.get_brawlers().len().to_string(),
                    true,
                ),
            ])
            .timestamp(ctx.created_at())
            .color(0x0000FF)
    };
    ctx.prompt(msg, embed, None).await?;
    Ok(())
}

async fn deregister(ctx: &BotContext<'_>, msg: &ReplyHandle<'_>) -> Result<(), BotError> {
    let discord_id = ctx.author().id.to_string();
    let embed = CreateEmbed::new()
        .title("Deregister Profile")
        .description("Are you sure you want to deregister?")
        .color(0xFF0000);
    match ctx.confirmation(msg, embed).await? {
        true => {
            ctx.data().database.delete_user(&discord_id).await?;
            ctx.log(
                "Deregistration",
                format!("User {} has deregistered their profile", ctx.author().name),
                log::State::SUCCESS,
                log::Model::PLAYER,
            )
            .await?;
        }
        false => {
            ctx.prompt(
                msg,
                CreateEmbed::new()
                .title("Deregistration (Cancelled)")
                .description("You have canceled deregistering your profile. This means you are still registered."),
        None
            ).await?;
        }
    }
    Ok(())
}

async fn leave_tournament(ctx: &BotContext<'_>, msg: &ReplyHandle<'_>) -> Result<(), BotError> {
    let discord_id = ctx.author().id.to_string();
    let tournaments = ctx
        .data()
        .database
        .get_active_tournaments_from_player(&discord_id)
        .await?;
    if tournaments.is_empty() {
        ctx.prompt(
            msg,
            CreateEmbed::new()
                .title("Leaving a tournament")
                .description("You are not in any tournament."),
            None,
        )
        .await?;
        return Ok(());
    }
    let selected_tournament_id = select_options(
        ctx,
        msg,
        "Leaving a tournament",
        "Select the tournament you want to leave",
        &tournaments,
    )
    .await?;
    let selected_tournament = tournaments
        .iter()
        .find(|t| t.tournament_id == selected_tournament_id.parse::<i32>().unwrap())
        .unwrap();
    let description = format!(
        r#"Confirm that you want to leave the following tournament:
Tournament name: {}"#,
        selected_tournament.name
    );
    let embed = CreateEmbed::new()
        .title("Leave Tournament")
        .description(description)
        .color(0xFF0000);
    match ctx.confirmation(msg, embed).await? {
        true => {
            ctx.data()
                .database
                .exit_tournament(&selected_tournament.tournament_id, &discord_id)
                .await?;
            ctx.prompt(
                msg,
                CreateEmbed::new()
                    .title("Leaving a tournament")
                    .description("You have successfully left the tournament."),
                None,
            )
            .await?;
        }
        false => {
            ctx.prompt(
                msg,
                CreateEmbed::new()
                    .title("Leaving a tournament (Cancelled)")
                    .description("You have canceled leaving the tournament. This means you are still in the tournament."),
        None
            ).await?;
        }
    }
    Ok(())
}
