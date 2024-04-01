use poise::{serenity_prelude as serenity, CreateReply};

use super::CommandsContainer;
use crate::{
    database::{Database, PgDatabase},
    tournament_model::SingleElimTournament,
    BotError, Context,
};

/// CommandsContainer for the Manager commands
pub struct ManagerCommands;

impl CommandsContainer<PgDatabase, SingleElimTournament> for ManagerCommands {
    fn get_commands_list(
    ) -> Vec<poise::Command<crate::BotData<PgDatabase, SingleElimTournament>, BotError>> {
        vec![set_config()]
    }
}

/// Set the configuration for a guild
///
/// This command is used to set the configuration for a guild. The configuration includes the marshal role, announcement channel, notification channel, and log channel.
///
/// Marshal Role: The role that is able to manage the tournament system. Akin to tournament
/// marshals.
/// Announcement Channel: The channel where the bot will announce the start and end of tournaments.
/// and other important announcements.
/// Notification Channel: The channel where the bot will send notifications to users about their
/// progress and matches.
/// Log Channel: The channel where the bot will log all the actions it takes.
#[poise::command(slash_command, prefix_command, guild_only)]
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
            return Ok(());
        }
    };

    let marshal_role_id = marshal_role.id.to_string();

    ctx.data()
        .database
        .set_config(
            ctx.guild_id().expect("Guild ID not found").to_string(),
            marshal_role_id,
            announcement_channel_id,
            notification_channel_id,
            log_channel_id,
        )
        .await?;

    ctx.send(
        CreateReply::default()
            .content("Successfully set the configuration. You can run the same command again to update the configuration.")
            .ephemeral(true),
    )
    .await?;

    Ok(())
}
