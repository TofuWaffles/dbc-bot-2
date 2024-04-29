use poise::{serenity_prelude as serenity, CreateReply};
use tracing::{error, info, instrument};

use crate::{commands::checks::is_manager, database::Database, BotData, BotError, Context};

use super::CommandsContainer;

/// CommandsContainer for the Manager commands
pub struct ManagerCommands;

impl CommandsContainer for ManagerCommands {
    type Data = BotData;
    type Error = BotError;

    fn get_commands_list() -> Vec<poise::Command<Self::Data, Self::Error>> {
        vec![set_config(), create_tournament()]
    }
}

/// Set the configuration for a guild
///
/// This command is used to set the configuration for a guild. The configuration includes the marshal role, announcement channel, notification channel, and log channel.
///
/// - Marshal Role: The role that is able to manage the tournament system. Akin to tournament
/// marshals.
/// - Announcement Channel: The channel where the bot will announce the start and end of tournaments.
/// and other important announcements.
/// - Notification Channel: The channel where the bot will send notifications to users about their
/// progress and matches.
/// - Log Channel: The channel where the bot will log all the actions it takes.
#[poise::command(slash_command, prefix_command, guild_only, check = "is_manager")]
#[instrument]
async fn set_config(
    ctx: Context<'_>,
    marshal_role: serenity::Role,
    announcement_channel: serenity::Channel,
    notification_channel: serenity::Channel,
    log_channel: serenity::Channel,
) -> Result<(), BotError> {
    let announcement_channel_id = match announcement_channel.guild() {
        Some(guild) => guild.id.to_string(),
        None => {
            ctx.send(
                CreateReply::default()
                    .content("Please enter a valid server channel as the announcement channel")
                    .ephemeral(true),
            )
            .await?;
            error!("Invalid announcement channel entered by {}", ctx.author());
            return Ok(());
        }
    };

    let notification_channel_id = match notification_channel.guild() {
        Some(guild) => guild.id.to_string(),
        None => {
            ctx.send(
                CreateReply::default()
                    .content("Please enter a valid server channel as the notification channel")
                    .ephemeral(true),
            )
            .await?;
            error!("Invalid notification channel entered by {}", ctx.author());
            return Ok(());
        }
    };

    let log_channel_id = match log_channel.guild() {
        Some(guild) => guild.id.to_string(),
        None => {
            ctx.send(
                CreateReply::default()
                    .content("Please enter a valid server channel as the log channel")
                    .ephemeral(true),
            )
            .await?;
            error!("Invalid log channel entered by {}", ctx.author());
            return Ok(());
        }
    };

    let marshal_role_id = marshal_role.id.to_string();

    ctx.data()
        .database
        .set_config(
            &ctx.guild_id()
                .ok_or("This command must be used within a server")?
                .to_string(),
            &marshal_role_id,
            &announcement_channel_id,
            &notification_channel_id,
            &log_channel_id,
        )
        .await?;

    ctx.send(
        CreateReply::default()
            .content("Successfully set the configuration. You can run the same command again to update the configuration.")
            .ephemeral(true),
    )
    .await?;

    info!(
        "Set the configuration for guild {}",
        ctx.guild_id().unwrap().to_string()
    );

    Ok(())
}

/// Create a new tournament.
#[poise::command(slash_command, prefix_command, guild_only, check = "is_manager")]
#[instrument]
async fn create_tournament(ctx: Context<'_>, name: String) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap().to_string();

    let new_tournament_id = ctx
        .data()
        .database
        .create_tournament(&guild_id, &name)
        .await?;

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Successfully created tournament with id {}",
                new_tournament_id
            ))
            .ephemeral(true),
    )
    .await?;

    info!(
        "Created tournament {} for guild {}",
        new_tournament_id, guild_id
    );

    Ok(())
}
