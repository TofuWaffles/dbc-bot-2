use crate::api::official_brawl_stars::MapEvent;
use crate::api::{images::ImagesAPI, official_brawl_stars::BattleLogItem};
use crate::commands::checks::is_tournament_paused;
use crate::database::models::Tournament;
use crate::database::models::{
    BattleRecord, BattleResult, BattleType, Match, Player, TournamentStatus,
};
use crate::database::{ConfigDatabase, MatchDatabase, TournamentDatabase, UserDatabase};
use crate::log::{self, Log};
use crate::mail::MailBotCtx;
use crate::utils::discord::{modal, select_options};
use crate::utils::error::CommonError::*;
use crate::utils::shorthand::BotContextExt;
use crate::{api::APIResult, commands::checks::is_config_set};
use crate::{BotContext, BotData, BotError};
use anyhow::anyhow;
use futures::Stream;
use poise::serenity_prelude::{futures::StreamExt, *};
use poise::{CreateReply, Modal, ReplyHandle};
use prettytable::{row, Table};
use serde_json::json;
use tokio::join;
use tracing::{info, instrument};

use super::CommandsContainer;

/// CommandsContainer for the User commands
pub struct UserCommands;

impl CommandsContainer for UserCommands {
    type Data = BotData;
    type Error = BotError;

    fn get_all() -> Vec<poise::Command<Self::Data, Self::Error>> {
        vec![menu(), credit()]
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
        .get_player_from_discord_id(ctx.author().id.to_string())
        .await?;
    let embed = CreateEmbed::new()
        .title("Menu")
        .description("Please wait while we load the menu.")
        .color(0x0000FF);
    let msg = ctx
        .send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    let interaction_collector = ctx.create_interaction_collector(&msg).await?;

    if let Some(user) = user {
        if !user.deleted {
            return user_display_menu(&ctx, &msg).await;
        }
    }

    ctx.prompt(
        &msg,
        CreateEmbed::new()
            .title("Registration Page Menu")
            .description("Loading registration page...")
            .color(Color::BLUE),
        None,
    )
    .await?;
    return user_display_registration(&ctx, &msg, interaction_collector).await;
}

/// Display the main menu to the registered user.
#[instrument(skip(msg))]
async fn user_display_menu(ctx: &BotContext<'_>, msg: &ReplyHandle<'_>) -> Result<(), BotError> {
    info!("User {} has entered the menu home", ctx.author().name);
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;
    ctx.mail_notification().await?;
    let mut player_active_tournaments = ctx
        .data()
        .database
        .get_player_active_tournaments(&guild_id, &ctx.author().id)
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
            CreateButton::new("mail")
                .label("Mail")
                .emoji(ReactionType::Unicode("📧".to_string()))
                .style(ButtonStyle::Primary),
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
        let mut buttons = vec![
            CreateButton::new("menu_match")
                .label("View Match")
                .style(ButtonStyle::Primary),
            CreateButton::new("profile")
                .label("Profile")
                .style(ButtonStyle::Primary),
            CreateButton::new("submit")
                .label("Submit")
                .style(ButtonStyle::Primary),
            CreateButton::new("mail")
                .label("Mail")
                .emoji(ReactionType::Unicode("📧".to_string()))
                .style(ButtonStyle::Primary),
        ];
        if player_active_tournaments[0].status == TournamentStatus::Pending {
            buttons.push(
                CreateButton::new("leave_tournament")
                    .label("Leave Tournament")
                    .style(ButtonStyle::Danger),
            );
        }
        ctx.prompt(msg, embed, buttons).await?;
    } else {
        return Err(anyhow!(
            "User {} with ID {} has enetered more than one active tournament",
            ctx.author().name,
            ctx.author().id,
        ));
    }
    let mut ic = ctx.create_interaction_collector(msg).await?;
    if let Some(interaction) = &ic.next().await {
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
                return leave_tournament(ctx, msg).await;
            }
            "submit" => {
                interaction.defer(ctx.http()).await?;
                let game_match = ctx
                    .data()
                    .database
                    .get_match_by_player(
                        player_active_tournaments[0].tournament_id,
                        &ctx.author().id,
                    )
                    .await?;
                return submit(
                    ctx,
                    msg,
                    &player_active_tournaments[0],
                    &game_match.unwrap(),
                )
                .await;
            }
            "mail" => {
                interaction.defer(ctx.http()).await?;
                if ctx.inbox(msg).await.is_err() {
                    msg.delete(*ctx).await?;
                }
            }
            _ => {}
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

    let current_match = match ctx
        .data()
        .database
        .get_current_match(&ctx.author().id)
        .await?
    {
        Some(m) => m,
        None => {
            ctx.prompt(
                msg,
                CreateEmbed::new().title("Match Not Found").description(
                    "You are not currently in a match. Please wait for the next round to begin.",
                ),
                None,
            )
            .await?;
            return Ok(());
        }
    };

    if !current_match.is_not_bye() {
        // Automatically advance the player to the next round if the opponent is a dummy
        // (a bye round)
        ctx.data()
            .database
            .set_winner(
                &current_match.match_id,
                &current_match
                    .match_players
                    .first()
                    .ok_or(anyhow!(
                        "Error displaying a bye round to user: No player found in match {}",
                        &current_match.match_id
                    ))?
                    .to_user(ctx)
                    .await?
                    .id,
                "bye",
            )
            .await?;
        ctx.prompt(msg,
            CreateEmbed::new().title("Match Information.")
            .description(
                "You have no opponents for the current round. See you in the next round, partner!",
            )
            .fields(vec![
                ("Tournament", &tournament.name, true),
                ("Match ID", &current_match.match_id, true),
                ("Round", &current_match.round()?.to_string(), true),
            ])
            , None).await?;

        return Ok(());
    }

    let player = current_match.get_player(&ctx.author().id.to_string())?;
    let discord_id = ctx.author().id;
    ctx.data().database.get_current_match(&discord_id).await?;
    let p1 = ctx
        .get_player_from_discord_id(None)
        .await?
        .ok_or(anyhow!("Player 1 is not found in the database"))?;
    let opp = current_match.get_opponent(&ctx.author().id.to_string())?;
    let p2 = ctx
        .get_player_from_discord_id(opp.discord_id.clone())
        .await?
        .ok_or(anyhow!("Player 2 is not found in the database"))?;

    let reply = {
        let image = ctx.data().apis.images.match_image(&p1, &p2).await?;
        let embed = {
            CreateEmbed::new()
                .title("Match Information.")
                .description(
                    "Here is all the information for your current match. May the best brawler win!",
                )
                .fields(vec![
                    ("Tournament", tournament.name.clone(), true),
                    ("Match ID", current_match.match_id.to_owned(), true),
                    ("Round", current_match.round()?.to_string(), true),
                    ("Game Mode", format!("{}", tournament.mode), true),
                    ("Map", tournament.map.name, true),
                    ("Wins required", tournament.wins_required.to_string(), true),
                    (
                        "Player 1",
                        current_match
                            .match_players
                            .first()
                            .ok_or(anyhow!(
                                "Error displaying player 1 for match {}: no player found",
                                current_match.match_id
                            ))?
                            .to_user(ctx)
                            .await?
                            .mention()
                            .to_string(),
                        false,
                    ),
                    (
                        "Player 2",
                        current_match
                            .match_players
                            .get(1)
                            .ok_or(anyhow!(
                                "Error displaying player 2 for match {}: no player found",
                                current_match.match_id
                            ))?
                            .to_user(ctx)
                            .await?
                            .mention()
                            .to_string(),
                        false,
                    ),
                ])
        };
        let buttons = {
            let mut buttons = vec![];
            buttons.push(
                CreateButton::new("mail")
                    .label("Mail")
                    .emoji(ReactionType::Unicode("📧".to_string()))
                    .style(ButtonStyle::Primary),
            );
            if !player.ready {
                buttons.push(
                    CreateButton::new("match_menu_ready")
                        .label("Ready")
                        .style(ButtonStyle::Success),
                );
            }
            if current_match.winner(ctx).await?.is_none() {
                buttons.push(
                    CreateButton::new("match_menu_forfeit")
                        .label("Forfeit")
                        .style(ButtonStyle::Danger),
                );
            }
            buttons
        };
        CreateReply::default()
            .attachment(CreateAttachment::bytes(image, "Match.png"))
            .embed(embed)
            .components(vec![CreateActionRow::Buttons(buttons)])
    };
    msg.edit(*ctx, reply).await?;
    let mut ic = ctx.create_interaction_collector(msg).await?;
    if let Some(interaction) = &ic.next().await {
        match interaction.data.custom_id.as_str() {
            "match_menu_ready" => {
                interaction.defer(ctx.http()).await?;
                ctx.prompt(
                msg,
                CreateEmbed::new()
                    .title("Ready Confirmation")
                    .description("You have set yourself to ready. A notification has been sent to your opponent to let them know.\n\nBe sure to play your matches and hit the \"Submit\" button when you're done.")
                    .color(Color::DARK_GREEN),
                None,
            ).await?;

                let player = ctx.author();
                let opponent = current_match.get_opponent(player.id.to_string().as_str())?;
                let opponent_user = opponent.to_user(ctx).await?;

                ctx.data()
                    .database
                    .set_ready(&current_match.match_id.clone(), &player.id)
                    .await?;

                let notification_message = if opponent.ready {
                    format!(
                        r#"{}-{}.\n\nBoth players are ready to battle. Please complete your matches and press the "Submit" button once you're finished. Good luck to both of you!"#,
                        player.mention(),
                        opponent_user.mention()
                    )
                } else {
                    format!(
                        r#"Hey {}, your opponent {} is ready for battle. Let us know when you're ready by clicking the ready button in the menu (type /menu to open the menu). See you on the battlefield!"#,
                        opponent.to_user(ctx).await?.mention(),
                        player.mention()
                    )
                };

                let notification_channel =
                    ChannelId::new(tournament.notification_channel_id.parse()?);
                notification_channel
                    .send_message(ctx, CreateMessage::default().content(notification_message))
                    .await?;
            }
            "mail" => {
                interaction.defer(ctx.http()).await?;
                ctx.compose(
                    msg,
                    p2.user(ctx).await?.id,
                    current_match.clone().match_id.clone(),
                )
                .await?;
            }
            "match_menu_forfeit" => {
                interaction.defer(ctx.http()).await?;
                user_forfeit(ctx, msg, current_match).await?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[instrument(skip(msg))]
async fn user_forfeit(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    bracket: Match,
) -> Result<(), BotError> {
    let forfeit = ctx.confirmation(msg, CreateEmbed::new()
                     .title("⚠️Forfeit Match⚠️")
                     .description("Warning: Forfeiting the match means that you will drop out of the tournament and your opponent will automatically win. This action is NOT reversable.\n\nAre you sure you want to continue?"))
                    .await?;

    if forfeit {
        let opponent = bracket.get_opponent(&ctx.author().id.to_string())?;
        ctx.data()
            .database
            .set_winner(
                &bracket.match_id,
                &opponent.to_user(ctx).await?.id,
                &bracket.score,
            )
            .await?;
        msg.edit(
            *ctx,
            CreateReply::default()
                .content("You've successfully forfeited the match. Hope to see you in the next tournament, partner!")
                .ephemeral(true),
        )
        .await?;
    } else {
        msg.edit(
            *ctx,
            CreateReply::default()
                .content("Cancelled forfeiting the match.")
                .ephemeral(true),
        )
        .await?;
    }

    return Ok(());
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
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;
    let tournaments: Vec<Tournament> = ctx
        .data()
        .database
        .get_active_tournaments(&guild_id)
        .await?
        .into_iter()
        .filter(|tournament| tournament.is_pending())
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
        loop {
            let selected = select_options(
                ctx,
                msg,
                CreateEmbed::default()
                    .title("Tournament Enrollment")
                    .description(
                        "Here are all the active tournaments in this server.\n\nTo join a tournament, click the button with the number corresponding to the one you wish to join.",
                    ),
                    None,
               &tournaments
            ).await?;
            let name = tournaments
                .iter()
                .find(|t| t.tournament_id == selected.parse::<i32>().unwrap())
                .unwrap()
                .name
                .clone();
            let description = format!(
                r#"Please confirm that you want to participate in the following tournament
{}"#,
                name
            );
            let embed = CreateEmbed::new()
                .title("Tournament Enrollment")
                .description(description);
            if ctx.confirmation(msg, embed).await? {
                break selected;
            }
        }
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
        .enter_tournament(selected_tournament.parse::<i32>()?, &ctx.author().id)
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
    let mut user = Player::default();
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
        interaction
            .create_response(ctx, CreateInteractionResponse::Acknowledge)
            .await?;
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
    let api_result = ctx
        .data()
        .apis
        .brawl_stars
        .get_player(&user.player_tag)
        .await?;
    match api_result {
        APIResult::Ok(player) => {
            let embed = {
                CreateEmbed::new()
                    .title(format!("**{} ({})**", player.name, player.tag))
                    .description("**Please confirm that this is your profile**")
                    .thumbnail(player.icon())
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
        APIResult::NotFound => {
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
        APIResult::Maintenance => {
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
    let player = match ctx
        .get_player_from_discord_id(ctx.author().id.to_string())
        .await?
    {
        Some(player) => player,
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
    display_user_profile_helper(ctx, msg, player).await?;
    Ok(())
}

async fn display_user_profile_helper(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    user: Player,
) -> Result<(), BotError> {
    let tournament = ctx
        .data()
        .database
        .get_active_tournaments_from_player(&ctx.author().id)
        .await?
        .first()
        .cloned();
    let tournament_id = tournament
        .as_ref()
        .map_or_else(|| "None".to_string(), |t| t.tournament_id.to_string());
    let image_api = ImagesAPI::new();
    let image = image_api
        .profile_image(&user, tournament_id.to_string())
        .await?;
    let reply = {
        let embed = CreateEmbed::new()
            .title("Match image")
            .author(ctx.get_author_img(&log::Model::PLAYER))
            .description("Here is some of your data")
            .color(Color::DARK_GOLD)
            .fields(vec![(
                "Tournament",
                tournament.map_or_else(|| "None".to_string(), |t| t.name.clone()),
                true,
            )]);
        CreateReply::default()
            .reply(true)
            .embed(embed)
            .attachment(CreateAttachment::bytes(image, "profile_image.png"))
    };
    msg.edit(*ctx, reply).await?;
    Ok(())
}

#[poise::command(context_menu_command = "User Profile")]
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
            ctx.prompt(
                &msg,
                CreateEmbed::new()
                    .title("Profile Not Found")
                    .description("The user has not registered their profile yet. Please run the /menu command to register their profile."),
                None
            ).await?;
            return Ok(());
        }
        Err(e) => {
            ctx.prompt(
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

async fn deregister(ctx: &BotContext<'_>, msg: &ReplyHandle<'_>) -> Result<(), BotError> {
    let discord_id = ctx.author().id;
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
    let discord_id = ctx.author().id;
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
        CreateEmbed::default()
            .title("Leaving a tournament")
            .description("Select the tournament you want to leave"),
        None,
        &tournaments,
    )
    .await?;
    let selected_tournament = tournaments
        .iter()
        .find(|t| t.tournament_id == selected_tournament_id.parse::<i32>().unwrap())
        .ok_or(anyhow!("The tournament with id {} was not found in the list of the player's tournaments when player tried to leave.", selected_tournament_id))?;
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

async fn submit(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    tournament: &Tournament,
    bracket: &Match,
) -> Result<(), BotError> {
    async fn filter(
        ctx: &BotContext<'_>,
        logs: Vec<BattleLogItem>,
        game_match: &Match,
        tournament: &Tournament,
    ) -> Result<Vec<BattleLogItem>, BotError> {
        let [p1, p2] = match &game_match.match_players[..] {
            [p1, p2] => [
                ctx.get_player_from_discord_id(p1.discord_id.to_string())
                    .await?
                    .ok_or_else(|| {
                        anyhow!("Cannot find player with the Discord {}", p1.discord_id)
                    })?,
                ctx.get_player_from_discord_id(p2.discord_id.to_string())
                    .await?
                    .ok_or_else(|| {
                        anyhow!("Cannot find player with the Discord {}", p2.discord_id)
                    })?,
            ],
            _ => {
                return Err(anyhow!(
                    "Error submitting results for match {}: unable to find both players",
                    game_match.match_id
                ))
            }
        };
        let mut tags = vec![p1.player_tag.clone(), p2.player_tag.clone()];
        tags.sort();
        let compare_tag = |s1: &str, s2: &str| {
            s1.chars()
                .zip(s2.chars())
                .all(|(c1, c2)| c1 == c2 || (c1 == 'O' && c2 == '0') || (c1 == '0' && c2 == 'O'))
                && s1.len() == s2.len()
        };
        let filtered_logs = logs
            .iter()
            .filter(|log| {
                tournament.map.eq(&(log.event.clone().into(0).map))
                    && (log.battle.mode.eq(&tournament.mode) || log.event.mode.eq(&tournament.mode))
                    && log
                        .battle
                        .battle_type
                        .to_lowercase()
                        .eq(&BattleType::friendly.to_string().to_lowercase())
                    && {
                        let mut log_tags = vec![
                            format!("{}", &log.battle.teams[0][0].tag),
                            format!("{}", &log.battle.teams[1][0].tag),
                        ];
                        log_tags.sort();
                        tags.iter()
                            .zip(log_tags.iter())
                            .all(|(tag1, tag2)| compare_tag(tag1, tag2))
                    }
            })
            .cloned()
            .collect::<Vec<BattleLogItem>>();
        Ok(filtered_logs)
    }
    /// Analyse the battle logs to determine the winner of the match
    /// Returns true if the command caller wins, false if the opponent wins, and None if no conclusion can be made
    async fn analyze(tournament: &Tournament, battles: &[BattleLogItem]) -> Option<(bool, String)> {
        let mut conclusion: Option<(bool, String)> = None; //true = player 1, false = player 2, None = no conclusion
        let mut victory = 0;
        let mut defeat = 0;
        let results = battles
            .iter()
            .map(|b| b.battle.result)
            .collect::<Vec<BattleResult>>();
        for result in results {
            match result {
                BattleResult::victory => victory += 1,
                BattleResult::defeat => defeat += 1,
                _ => {}
            }
            if defeat == tournament.wins_required && victory < tournament.wins_required {
                conclusion = Some((false, format!("{}-{}", defeat, victory)));
                break;
            } else if victory >= tournament.wins_required {
                conclusion = Some((true, format!("{}-{}", victory, defeat)));
                break;
            }
        }
        conclusion
    }
    async fn handle_not_enough_matches(
        ctx: &BotContext<'_>,
        msg: &ReplyHandle<'_>,
    ) -> Result<(), BotError> {
        ctx.prompt(
            msg,
            CreateEmbed::new()
                .title("Insufficient Matches")
                .description("You have not played enough matches to submit. You need to play at least 3 matches to submit."),
            None,
        )
        .await?;
        ctx.log(
            "Insufficient Matches",
            format!(
                "User {} has not played enough matches to submit",
                ctx.author().name
            ),
            crate::log::State::FAILURE,
            crate::log::Model::PLAYER,
        )
        .await?;
        Ok(())
    }

    async fn save_record(
        ctx: &BotContext<'_>,
        game_match: &Match,
        battles: Vec<BattleLogItem>,
    ) -> Result<(), BotError> {
        let match_id = game_match.match_id.clone();
        let record = BattleRecord::new(ctx, match_id, battles);
        record.execute(ctx).await?;
        Ok(())
    }
    let caller = ctx.author().id;
    let current_match = match ctx.data().database.get_current_match(&caller).await? {
        Some(m) => m,
        None => {
            ctx.prompt(
                msg,
                CreateEmbed::new().title("Match Not Found").description(
                    "You are not currently in a match. Please wait for the next round to begin.",
                ),
                None,
            )
            .await?;
            return Ok(());
        }
    };

    let caller_tag = ctx
        .get_player_from_discord_id(caller.get().to_string())
        .await?
        .ok_or(anyhow!("Player not found in the database"))?
        .player_tag;
    let logs = match ctx
        .data()
        .apis
        .brawl_stars
        .get_battle_log(&caller_tag)
        .await?
    {
        APIResult::Ok(response) => response.items,
        APIResult::NotFound => {
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
                format!("Player tag {} not found", caller_tag),
                crate::log::State::FAILURE,
                crate::log::Model::PLAYER,
            )
            .await?;
            return Ok(());
        }
        APIResult::Maintenance => {
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
            return Ok(());
        }
    };
    ctx.prompt(
        msg,
        CreateEmbed::new()
            .title("Analyzing results")
            .description("Hold on. I am analyzing the battle records..."),
        None,
    )
    .await?;
    let battles = filter(ctx, logs, &current_match, tournament).await?;
    if battles.len() < tournament.wins_required as usize {
        return handle_not_enough_matches(ctx, msg).await;
    }
    let winner = analyze(tournament, &battles).await;
    let score = winner.clone().map(|(_, s)| s).unwrap_or("0-0".to_string());
    let target = match winner {
        None => return handle_not_enough_matches(ctx, msg).await,
        Some((true, score)) => join!(
            ctx.data()
                .database
                .set_winner(&current_match.match_id, &caller, &score),
            ctx.get_player_from_discord_id(None)
        )
        .1?
        .ok_or(anyhow!("Player not found in the database"))?,
        Some((false, score)) => {
            let opponent_id = &bracket
                .get_opponent(&ctx.author().id.to_string())?
                .to_user(ctx)
                .await?;
            join!(
                ctx.data()
                    .database
                    .set_winner(&bracket.match_id, &opponent_id.id, &score),
                ctx.get_player_from_discord_id(opponent_id.id.to_string())
            )
            .1?
            .ok_or(anyhow!("Player not found in the database"))?
        }
    };

    let (adv, elim) = (
        &target,
        ctx.get_player_from_discord_id(
            current_match
                .get_opponent(&target.discord_id)?
                .discord_id
                .clone(),
        )
        .await?
        .unwrap(),
    );
    save_record(ctx, &current_match, battles).await?;
    let (image, user) = join!(
        ctx.data()
            .apis
            .images
            .clone()
            .result_image(adv, &elim, &score),
        target.user(ctx)
    );
    // Final round. Announce the winner and finish the tournament
    if bracket.round()? == tournament.rounds {
        finish_tournament(ctx, bracket, &image?, &target).await?;
        return Ok(());
    }

    let embed = CreateEmbed::new()
        .title("Match submission!")
        .description(format!(
            "Congratulations! {} passes Round {}",
            user?.mention(),
            tournament.current_round
        ))
        .thumbnail(target.icon());
    let channel = tournament.notification_channel(ctx).await?;

    let result_msg = channel
        .send_message(
            ctx.http(),
            CreateMessage::new()
                .embed(embed)
                .add_file(CreateAttachment::bytes(image?, "result.png")),
        )
        .await?;
    ctx.prompt(
        msg,
        CreateEmbed::new()
            .title("Result has been recorded successfully!")
            .description(format!(
                "Click [here]({}) to see the result\nOr head to {} to view other results!",
                result_msg.link(),
                channel.mention()
            )),
        None,
    )
    .await?;
    ctx.log(
        "Match submission",
        format!(
            "User {} has submitted their match {}",
            ctx.author().name,
            bracket.match_id
        ),
        log::State::SUCCESS,
        log::Model::PLAYER,
    )
    .await?;
    Ok(())
}

async fn finish_tournament(
    ctx: &BotContext<'_>,
    bracket: &Match,
    image: &[u8],
    winner: &Player,
) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;
    let announcement_channel_id = ctx
        .data()
        .database
        .get_config(&guild_id)
        .await?
        .ok_or(anyhow!("Config not found for {}", guild_id.to_string()))?
        .announcement_channel_id;
    let tournament_id = bracket.tournament()?;
    let tournament = ctx
        .data()
        .database
        .get_tournament(&guild_id, tournament_id)
        .await?
        .ok_or(TournamentNotExists(tournament_id.to_string()))?;

    let reply = {
        let embed = CreateEmbed::new()
            .title("Tournament Finished!")
            .description(format!(
                "Congratulations to <@{}> for winning Tournament {}",
                winner.discord_id, tournament.name
            ))
            .thumbnail(winner.icon())
            .color(Color::DARK_GREEN);
        CreateMessage::default()
            .embed(embed)
            .add_file(CreateAttachment::bytes(image, "result.png"))
    };

    ChannelId::new(announcement_channel_id.parse::<u64>()?)
        .send_message(ctx, reply)
        .await?;

    ctx.data()
        .database
        .set_tournament_status(tournament_id, TournamentStatus::Inactive)
        .await?;

    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    check = "is_config_set",
    check = "is_tournament_paused"
)]
#[instrument]
async fn credit(ctx: BotContext<'_>) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default()
                .embed(
                    CreateEmbed::new()
                        .title("Credit")
                        .description("Loading credit..."),
                )
                .reply(true)
                .ephemeral(true),
        )
        .await?;
    let description = "";

    ctx.prompt(
        &msg,
        CreateEmbed::new().title("Credit").description(description),
        None,
    )
    .await?;
    Ok(())
}
