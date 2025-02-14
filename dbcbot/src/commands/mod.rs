pub mod manager_commands;
pub mod marshal_commands;
pub mod owner_commands;
pub mod test_commands;
pub mod user_commands;

/// A way to group commands together while side-stepping the need to use global variables.
///
/// Implemenations of this trait can return a list of their commands from within their own module.
/// Typically, you would group commands by their required permissions or role.
///
/// Additionally, the implementations of this trait should not directly "own" the commands.
/// You should simply generate them by running the functions generated by the [`poise::command`](https://docs.rs/poise/latest/poise/macros/attr.command.html) macro and returning the vector.
/// This way, you only need to `pub` the implementation and not the commands themselves.
///
/// For more details on how this works, you can check the documentation for the [`poise::command`](https://docs.rs/poise/latest/poise/macros/attr.command.html#internals) macro.
///
/// For example, you can define a type to group Manager commands together.
/// ```
/// pub struct ManagerCommands; // This struct doesn't need to hold anything
///
/// impl CommandsContainer<PgDatabase, SingleElimTournament> for ManagerCommands {
///     type Data = BotData;
///     type Error = BotError;
///
///     fn get_commands_list(&self)
///         -> Vec<poise::Command<Self::Data, Self::Error>> {
///         vec![very_important_manager_only_command()]
///     }
/// }
///
/// #[poise::command(slash_command, prefix_command, check = "is_manager")]
/// async fn very_important_manager_only_command(
///     ctx: Context<'_>,
/// ) -> Result<(), BotError> {
///     ctx.say("Wow, you're a manager, that's so cool!").await?;
///     Ok(())
/// }
/// ```
pub trait CommandsContainer {
    type Data;
    type Error;

    /// Retrive all the commands from a module, such as manager commands or marshal commands.
    fn get_all() -> Vec<poise::Command<Self::Data, Self::Error>>;
}

/// Common checks (e.g. role checks) used by various commands.
mod checks {
    use crate::utils::error::CommonError::*;
    use std::str::FromStr;

    use poise::{serenity_prelude::RoleId, CreateReply};

    use crate::{
        database::{models::TournamentStatus, ConfigDatabase, TournamentDatabase},
        BotContext, BotError,
    };

    /// Checks if the user has a manager role.
    pub async fn is_manager(ctx: BotContext<'_>) -> Result<bool, BotError> {
        let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;

        let member = guild_id.member(ctx, ctx.author().id).await?;

        if ctx
            .guild()
            .ok_or(NotInAGuild)?
            .member_permissions(&member)
            .administrator()
        {
            return Ok(true);
        }

        let manager_role_option = ctx.data().database.get_manager_role(&guild_id).await?;

        let manager_role_id = match manager_role_option {
            Some(manager_role) => RoleId::from_str(&manager_role.manager_role_id)?.get(),
            None => {
                ctx.send(
                    CreateReply::default()
                        .content("The manager role has not been set up for this server. Please ask the bot owner to set it up.")
                        .ephemeral(true),
                ).await?;
                return Ok(false);
            }
        };

        if ctx
            .author()
            .has_role(ctx, guild_id, manager_role_id)
            .await?
        {
            return Ok(true);
        }

        ctx.send(
            CreateReply::default()
                .content("You do not have the required permissions to use this command.")
                .ephemeral(true),
        )
        .await?;

        Ok(false)
    }

    /// Checks if the user is a marshal or higher (usually means manager or marshal role)
    pub async fn is_marshal_or_higher(ctx: BotContext<'_>) -> Result<bool, BotError> {
        let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;

        let member = guild_id.member(ctx, ctx.author().id).await?;

        if ctx
            .guild()
            .ok_or(NotInAGuild)?
            .member_permissions(&member)
            .administrator()
        {
            return Ok(true);
        }

        let manager_role = ctx.data().database.get_manager_role(&guild_id).await?;

        let marshal_role = ctx.data().database.get_config(&guild_id).await?;

        if manager_role.is_none() || marshal_role.is_none() {
            ctx.send(
                CreateReply::default()
                    .content("Either the manager role or the bot configuration has not been set up for this server. Please ask a bot owner to set it up.")
                    .ephemeral(true),
            )
            .await?;
            return Ok(false);
        }
        let manager_role_id = RoleId::from_str(&manager_role.unwrap().manager_role_id)?.get();

        let marshal_role_id = RoleId::from_str(&marshal_role.unwrap().marshal_role_id)?.get();

        if ctx
            .author()
            .has_role(ctx, guild_id, manager_role_id)
            .await?
            || ctx
                .author()
                .has_role(ctx, guild_id, marshal_role_id)
                .await?
        {
            return Ok(true);
        }

        ctx.send(
            CreateReply::default()
                .content("You do not have the required permissions to use this command.")
                .ephemeral(true),
        )
        .await?;

        Ok(false)
    }

    /// Checks if the configuration has been set up for the guild.
    pub async fn is_config_set(ctx: BotContext<'_>) -> Result<bool, BotError> {
        let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;

        let config = ctx.data().database.get_config(&guild_id).await?;

        if config.is_some() {
            return Ok(true);
        }

        ctx.send(
            CreateReply::default()
                .content("The bot configuration has not been set up for this server. Please ask a moderator to set it up.")
                .ephemeral(true),
        )
        .await?;

        Ok(false)
    }

    /// Check if the tournament that the user is in is paused.
    ///
    /// The check still returns true if the user is not in a tournament.
    pub async fn is_tournament_paused(ctx: BotContext<'_>) -> Result<bool, BotError> {
        let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;

        let tournaments = ctx
            .data()
            .database
            .get_player_active_tournaments(&guild_id, &ctx.author().id)
            .await?;

        match tournaments.first() {
            Some(tournament) => {
                if tournament.status == TournamentStatus::Paused {
                    ctx.send(CreateReply::default().content("Your tournament is currently paused. Please come back again later.").ephemeral(true)).await?;
                    return Ok(false);
                }
            }
            None => return Ok(true),
        }

        Ok(true)
    }
}
