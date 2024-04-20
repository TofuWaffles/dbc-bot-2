use std::time::Duration;

use poise::{
    serenity_prelude::{
        futures::StreamExt, ButtonStyle, CreateActionRow, CreateButton, CreateEmbed,
    },
    CreateReply, ReplyHandle,
};
use prettytable::{row, Table};
use uuid::Uuid;

use crate::{
    api::{ApiResult, GameApi},
    database::Database,
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

#[poise::command(slash_command, prefix_command, guild_only)]
async fn menu(ctx: Context<'_>) -> Result<(), BotError> {
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

async fn user_display_menu(ctx: Context<'_>, msg: ReplyHandle<'_>) -> Result<(), BotError> {
    msg.edit(
        ctx,
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("Main Menu")
                    .description("Welcome to the menu! What would you like to do?"),
            )
            .components(vec![CreateActionRow::Buttons(vec![
                CreateButton::new("menu_tournaments")
                    .label("Tournaments")
                    .style(ButtonStyle::Primary),
                CreateButton::new("something_else")
                    .label("Something Else")
                    .style(ButtonStyle::Danger),
            ])]),
    )
    .await?;

    Ok(())
}

async fn user_display_tournaments(ctx: Context<'_>, msg: ReplyHandle<'_>) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let tournaments = ctx
        .data()
        .database
        .get_active_tournaments(&guild_id)
        .await?;

    let mut table = Table::new();
    table.set_titles(row!["Tournament ID", "Name", "Started"]);

    let mut buttons = Vec::new();

    for (i, tournament) in tournaments.iter().enumerate() {
        table.add_row(row![i, &tournament.name, &tournament.started]);

        buttons.push(
            CreateButton::new(format!("tournament_{}", i.to_string()))
                .label(i.to_string())
                .style(ButtonStyle::Primary),
        );
    }

    msg.edit(
        ctx,
        CreateReply::default()
            .embed(
                CreateEmbed::new()
                    .title("Tournaments")
                    .description("Here are the tournaments you are currently in:"),
            )
            .components(vec![CreateActionRow::Buttons(vec![
                CreateButton::new("menu_tournaments")
                    .label("Back")
                    .style(ButtonStyle::Primary),
                CreateButton::new("something_else")
                    .label("Something Else")
                    .style(ButtonStyle::Danger),
            ])]),
    )
    .await?;

    Ok(())
}

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
                                    ("Club", player.club.name, true),
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
