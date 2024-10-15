use poise::serenity_prelude as serenity;
use tracing::{info, instrument};
use crate::utils::error::CommonError::*;
use super::CommandsContainer;
use crate::database::ConfigDatabase;
use crate::{BotContext, BotData, BotError};

pub struct OwnerCommands;

/// CommandsContainer for the User commands
impl CommandsContainer for OwnerCommands {
    type Data = BotData;
    type Error = BotError;

    fn get_all() -> Vec<poise::Command<Self::Data, Self::Error>> {
        vec![set_manager()]
    }
}

/// Set the Manager role for the server. Only usable by the bot owner.
///
/// Managers are typically server moderators and are able to run any command except this one.
#[poise::command(slash_command, prefix_command, guild_only, owners_only)]
#[instrument]
async fn set_manager(
    ctx: BotContext<'_>,
    #[description = "The Manager role"] role: serenity::Role,
) -> Result<(), BotError> {

    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;
    let manager_role_id = role.id;

    ctx.data()
        .database
        .set_manager_role(&guild_id, &manager_role_id)
        .await?;

    ctx.send(
        poise::CreateReply::default()
            .content(format!(
                "Successfully set the manager role to {role_name}.",
                role_name = role.name
            ))
            .ephemeral(true),
    )
    .await?;

    info!(
        "Set the manager role for guild {} to {}",
        guild_id, manager_role_id
    );

    Ok(())
}
