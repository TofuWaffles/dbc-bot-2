use std::time::Duration;

use poise::{
    serenity_prelude::{
        futures::StreamExt, ButtonStyle, CreateActionRow, CreateButton, CreateEmbed,
    },
    CreateReply, ReplyHandle,
};
use prettytable::{row, Table};
use tracing::{error, info, instrument};
use uuid::Uuid;

use crate::{
    api::{ApiResult, GameApi},
    database::{models::Tournament, Database},
    reminder::MatchReminder,
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
const USER_CMD_TIMEOUT: u64 = 5000;

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

    match user {
        Some(_) => {
            user_display_menu(ctx, msg).await?;
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

#[instrument(skip(msg))]
async fn user_display_menu(ctx: Context<'_>, msg: ReplyHandle<'_>) -> Result<(), BotError> {
    info!("User {} has entered the menu home", ctx.author().name);
    let mut player_active_tournaments = ctx
        .data()
        .database
        .get_player_active_tournaments(
            &ctx.guild_id().unwrap().to_string(),
            &ctx.author().id.to_string(),
        )
        .await?;

    msg.edit(
        ctx,
        CreateReply::default()
            .content("")
            .embed(
                CreateEmbed::new()
                    .title("Main Menu")
                    .description("Welcome to the menu! What would you like to do?"),
            )
            .components(vec![CreateActionRow::Buttons(vec![
                if player_active_tournaments.len() < 1 {
                    CreateButton::new("menu_tournaments")
                        .label("Tournaments")
                        .style(ButtonStyle::Primary)
                } else if player_active_tournaments.len() == 1 {
                    CreateButton::new("menu_match")
                        .label("View Match")
                        .style(ButtonStyle::Success)
                } else {
                    panic!(
                        "User {} with ID {} has enetered more than one active tournament",
                        ctx.author().name,
                        ctx.author().id,
                    )
                },
                CreateButton::new("something_else")
                    .label("Something Else")
                    .style(ButtonStyle::Danger),
            ])]),
    )
    .await?;

    let mut interaction_collector = msg
        .clone()
        .into_message()
        .await?
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(Duration::from_millis(USER_CMD_TIMEOUT))
        .stream();

    while let Some(interaction) = &interaction_collector.next().await {
        match interaction.data.custom_id.as_str() {
            "menu_tournaments" => {
                msg.edit(
                    ctx,
                    CreateReply::default()
                        .content("Loading tournaments...")
                        .ephemeral(true),
                )
                .await?;
                user_display_tournaments(ctx, msg).await?;
                break;
            }
            "menu_match" => {
                msg.edit(
                    ctx,
                    CreateReply::default()
                        .content("Loading your match...")
                        .ephemeral(true),
                )
                .await?;
                user_display_match(ctx, msg, player_active_tournaments.remove(0)).await?;
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

#[instrument(skip(msg))]
async fn user_display_match(
    ctx: Context<'_>,
    msg: ReplyHandle<'_>,
    tournament: Tournament,
) -> Result<(), BotError> {
    info!(
        "User {} is viewing their entered tournament",
        ctx.author().name
    );

    msg.edit(
        ctx,
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("Here is your current tournament")
                    .fields(vec![
                        ("Name", tournament.name, true),
                        ("Tournament ID", tournament.tournament_id.to_string(), true),
                    ]),
            )
            .components(vec![])
            .ephemeral(true),
    )
    .await?;

    // Todo: Add actual match information here along with scheduling options

    Ok(())
}

#[instrument(skip(msg))]
async fn user_display_tournaments(ctx: Context<'_>, msg: ReplyHandle<'_>) -> Result<(), BotError> {
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

    let mut interaction_collector = msg
        .clone()
        .into_message()
        .await?
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(Duration::from_millis(USER_CMD_TIMEOUT))
        .stream();

    while let Some(interaction) = &interaction_collector.next().await {
        match interaction_ids.iter().position(|id| id == interaction.data.custom_id.as_str()) {
            Some(tournament_number) => {
                ctx.data().database.enter_tournament(&tournaments[tournament_number].tournament_id, &ctx.author().id.to_string()).await?;
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

            // We might wanna look into how expensive these clones are, but it's not too important
            // right now
            let mut interaction_collector = msg
                .clone()
                .into_message()
                .await?
                .await_component_interaction(&ctx.serenity_context().shard)
                .timeout(Duration::from_millis(USER_CMD_TIMEOUT))
                .stream();

            while let Some(interaction) = &interaction_collector.next().await {
                match interaction.data.custom_id.as_str() {
                    "confirm_register" => {
                        interaction.defer(ctx).await?;
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
                        interaction.defer(ctx).await?;
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

/// Used for match reminders; WIP
#[poise::command(slash_command, prefix_command, guild_only)]
async fn reminder(ctx: Context<'_>, duration: i32) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let config = ctx.data().database.get_config(&guild_id).await?.unwrap();

    let match_reminder = MatchReminder::new(
        Uuid::new_v4(),
        duration.to_string(),
        "789".to_string(),
        guild_id,
        config.notification_channel_id,
        chrono::offset::Utc::now(),
    );

    ctx.data()
        .match_reminders
        .lock()
        .await
        .insert_reminder(match_reminder)?;

    Ok(())
}
