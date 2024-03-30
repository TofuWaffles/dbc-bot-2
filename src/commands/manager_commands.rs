use poise::serenity_prelude as serenity;

use crate::{commands::CommandsContainer, database::PgDatabase, tournament_model::SingleElimTournament, BotError, Context};

/// CommandsContainer for the Manager commands
pub struct ManagerCommands;

impl CommandsContainer<PgDatabase, SingleElimTournament<PgDatabase>> for ManagerCommands {
    fn get_commands_list() -> Vec<poise::Command<crate::BotData<PgDatabase, SingleElimTournament<PgDatabase>>, BotError>> {
        vec![age()]
    }
}

/// Displays your or another user's account creation date
///
/// Used as a demo command for now
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), BotError> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}
