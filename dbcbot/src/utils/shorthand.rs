use super::discord::prompt;
use crate::{
    database::{models::GuildConfig, Database},
    BotContext, BotError,
};
use anyhow::anyhow;
use poise::serenity_prelude::{Colour, User};
use poise::ReplyHandle;
pub trait BotContextExt<'a> {
    async fn prompt<S>(
        &self,
        msg: impl Into<Option<ReplyHandle<'a>>>,
        title: S,
        description: S,
        color: impl Into<Option<Colour>>,
    ) -> Result<ReplyHandle<'a>, BotError>
    where
        S: Into<String> + Send + 'static;
    async fn get_config(&self) -> Result<Option<GuildConfig>, BotError>;
    async fn get_player_from_discord_id(
        &self,
        user: impl Into<Option<User>>,
    ) -> Result<Option<crate::database::models::Player>, BotError>;
    async fn get_player_from_tag(
        &self,
        tag: &str,
    ) -> Result<Option<crate::database::models::Player>, BotError>;
    async fn get_current_round(&self, tournament_id: i32) -> Result<i32, BotError>;

    fn now(&self) -> poise::serenity_prelude::model::Timestamp;
}

impl<'a> BotContextExt<'a> for BotContext<'a> {
    /// Prompt the user with a message.
    /// # Arguments
    /// * `title` - The title of the message.
    /// * `description` - The description of the message.
    /// * `color` - The color of the message.
    /// # Returns
    /// Returns a `Result` containing a `()` if the message was sent successfully.
    /// # Errors
    /// Returns a `BotError` if there is an issue sending the message.
    ///
    async fn prompt<S>(
        &self,
        msg: impl Into<Option<ReplyHandle<'a>>>,
        title: S,
        description: S,
        color: impl Into<Option<Colour>>,
    ) -> Result<ReplyHandle<'a>, BotError>
    where
        S: Into<String> + Send + 'static,
    {
        prompt(self, msg, title, description, color).await
    }
    /// Retrieves the configuration for the current guild.
    ///
    /// This function retrieves the configuration specific to the guild where the command
    /// is being executed.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing an optional `GuildConfig` object wrapped in an `Option`.
    ///
    /// # Errors
    ///
    /// Returns a `BotError` if:
    /// - The command is not executed within a guild context.
    /// - There is an issue with fetching the guild configuration from the database.
    async fn get_config(&self) -> Result<Option<GuildConfig>, BotError> {
        let guild_id = self
            .guild_id()
            .ok_or(anyhow!("Not running this in a guild"))?;
        let config = self
            .data()
            .database
            .get_config(&guild_id.to_string())
            .await?;
        Ok(config)
    }
    /// Get a player from a Discord user ID.
    ///
    /// # Arguments
    ///
    /// * `user` - An optional Discord `User` object. If provided, this user's ID will be used
    ///            to fetch the player. If `None`, the author's ID of the current context will
    ///            be used.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing an optional `Player` object wrapped in an `Option`, which if there exists a player with the given Discord ID, will be `Some(player)`. If no player exists with the given Discord ID, `None` will be returned.
    ///
    /// # Errors
    ///
    /// Returns a `BotError` if there is an issue with fetching the player from the database.
    async fn get_player_from_discord_id(
        &self,
        user: impl Into<Option<User>>,
    ) -> Result<Option<crate::database::models::Player>, BotError> {
        let id = match user.into() {
            Some(user) => user.id.to_string(),
            None => self.author().id.to_string(),
        };
        let player = self.data().database.get_player_by_discord_id(&id).await?;
        Ok(player)
    }

    /// Get a player from a tag.
    /// # Arguments
    /// * `tag` - The tag of the player to fetch.
    /// # Returns
    /// Returns a `Result` containing an optional `Player` object wrapped in an `Option`, which if there exists a player with the given tag, will be `Some(player)`. If no player exists with the given tag, `None` will be returned.
    /// # Errors
    /// Returns a `BotError` if there is an issue with fetching the player from the database.
    async fn get_player_from_tag(
        &self,
        tag: &str,
    ) -> Result<Option<crate::database::models::Player>, BotError> {
        let player = self.data().database.get_player_by_player_tag(tag).await?;
        Ok(player)
    }

    /// Get the current round of a tournament.
    ///
    /// # Arguments
    ///
    /// * `tournament_id` - The ID of the tournament to fetch the current round for.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the current round as an `i32`. If the tournament is not found,
    /// returns `0`.
    ///
    /// # Errors
    ///
    /// Returns a `BotError` if:
    /// - The command is not executed within a guild context.
    /// - There is an issue fetching the tournament data from the database.
    async fn get_current_round(&self, tournament_id: i32) -> Result<i32, BotError> {
        let guild_id = self
            .guild_id()
            .ok_or(anyhow!("Not running this command in a guild"))?;
        let round = self
            .data()
            .database
            .get_tournament(&guild_id.to_string(), tournament_id)
            .await?
            .map_or_else(|| 0, |t| t.current_round);
        Ok(round)
    }

    /// Get the current time in seconds since the Unix epoch.
    /// # Returns
    /// Returns the current time in seconds since the Unix epoch.
    /// # Example
    /// ```
    /// let now = ctx.now();
    /// ```
    /// # Note
    /// This function is useful for logging timestamps.
    fn now(&self) -> poise::serenity_prelude::model::Timestamp {
        self.created_at()
    }
}
