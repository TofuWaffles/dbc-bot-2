pub mod manager_commands;
pub mod owner_commands;

use crate::{database::Database, tournament_model::TournamentModel, BotData, BotError};

/// A way to group commands together.
///
/// Implementors of this trait can return a list of their commands within their own module.
/// Typically, you would group commands by their required permissions.
///
/// Additionally, the implementors of this trait should not directly "own" the commands.
/// You should simply generate them by running Poise's function and returning the result.
/// This way, you only need to `pub` the implementor and not the commands themselves.
///
/// For more details on how this works, you can check the documentation for the `poise::command` macro.
///
/// For example, you can define a type to group Manager commands together.
/// ```
/// pub struct ManagerCommands;
///
/// impl CommandsContainer<PgDatabase, SingleElimTournament<PgDatabase>> for ManagerCommands {
///     fn get_commands_list(&self) 
///         -> Vec<poise::Command<crate::BotData<PgDatabase, SingleElimTournament<PgDatabase>>, BotError>> {
///         vec![very_important_manager_only_command()]
///     }
/// }
///
/// #[poise::command(slash_command, prefix_command)]
/// async fn very_important_manager_only_command(
///     ctx: Context<'_>,
/// ) -> Result<(), BotError> {
///     ctx.say("Wow, you're a manager, that's so cool!").await?;
///     Ok(())
/// }
/// ```
pub trait CommandsContainer<DB, TM>
where
    DB: Database,
    TM: TournamentModel,
{
    fn get_commands_list() -> Vec<poise::Command<BotData<DB, TM>, BotError>>;
}
