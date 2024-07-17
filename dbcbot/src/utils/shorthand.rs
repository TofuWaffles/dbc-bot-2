use crate::{
    database::{models::GuildConfig, Database},
    BotContext, BotError,
};
use anyhow::anyhow;
use futures::{Stream, StreamExt};
use poise::ReplyHandle;
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteraction, CreateActionRow, CreateButton, CreateEmbed
    },
    CreateReply,
};
use tokio::time::Duration;
pub trait BotContextExt<'a> {
    /// Amount of time in milliseconds before message interactions (usually buttons) expire for user
    const USER_CMD_TIMEOUT: u64;
    async fn create_interaction_collector(
        &self,
        msg: &ReplyHandle<'_>,
    ) -> Result<impl Stream<Item = ComponentInteraction> + Unpin, BotError>;
    /// Prompt the user with a message.
    /// # Arguments
    /// * `msg` - An optional `ReplyHandle` object. If provided, this message will be edited on the provided one, If `None`, the message will be sent as a new message.
    /// * `title` - The title of the message.
    /// * `description` - The description of the message.
    /// * `color` - The color of the message.
    /// # Returns
    /// Returns a `Result` containing a `ReplyHandle` if the message was sent successfully.
    /// # Errors
    /// Returns a `BotError` if there is an issue sending the message.
    async fn prompt(
        &self,
        msg: &ReplyHandle<'a>,
        embed: CreateEmbed,
        buttons: impl Into<Option<Vec<CreateButton>>>,
    ) -> Result<(), BotError>;
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
    async fn get_config(&self) -> Result<Option<GuildConfig>, BotError>;

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
        discord_id: impl Into<Option<String>> + Clone,
    ) -> Result<Option<crate::database::models::Player>, BotError>;

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
    ) -> Result<Option<crate::database::models::Player>, BotError>;

    /// Get a user from a discord id.
    /// # Arguments
    /// * `discord_id` - The tag of the player to fetch.
    /// # Returns
    /// Returns a `Result` containing an optional `User` object wrapped in an `Option`, which if there exists a player with the given tag, will be `Some(User)`. If no player exists with the given tag, `None` will be returned.
    /// # Errors
    /// Returns a `BotError` if there is an issue with fetching the player from the database.
    async fn get_user_by_discord_id(
        &self,
        discord_id: impl Into<Option<String>> + Clone
    ) -> Result<Option<crate::database::models::User>, BotError>;
    /// 
    async fn get_current_round(&self, tournament_id: i32) -> Result<i32, BotError>;

    /// Prompt the user with a confirmation message.
    /// # Arguments
    /// * `msg` - The message to reply to.
    /// * `embed` - The embed to send with the confirmation message.
    /// # Returns
    /// Returns a `Result` containing a `bool` indicating whether the user confirmed the action.
    /// # Errors
    /// Returns a `BotError` if there is an issue sending the confirmation message.
    async fn confirmation(
        &self,
        msg: &ReplyHandle<'_>,
        embed: CreateEmbed,
    ) -> Result<bool, BotError>;

    /// Get the current time in seconds since the Unix epoch.
    /// # Returns
    /// Returns the current time in seconds since the Unix epoch.
    /// # Example
    /// ```
    /// let now = ctx.now();
    /// ```
    /// # Note
    /// This function is useful for logging timestamps.
    fn now(&self) -> poise::serenity_prelude::model::Timestamp;
}

impl<'a> BotContextExt<'a> for BotContext<'a> {
    const USER_CMD_TIMEOUT: u64 = 120_000;
    async fn create_interaction_collector(
        &self,
        msg: &ReplyHandle<'_>,
    ) -> Result<impl Stream<Item = ComponentInteraction> + Unpin, BotError> {
        let ic = msg
            .clone()
            .into_message()
            .await?
            .await_component_interaction(&self.serenity_context().shard)
            .timeout(Duration::from_millis(Self::USER_CMD_TIMEOUT))
            .stream();
        Ok(ic)
    }

    async fn prompt(
        &self,
        msg: &ReplyHandle<'a>,
        embed: CreateEmbed,
        buttons: impl Into<Option<Vec<CreateButton>>>,
    ) -> Result<(), BotError> {
        let components = match buttons.into(){
            Some(buttons) => vec![CreateActionRow::Buttons(buttons)],
            _ => vec![],
        };
        let builder = CreateReply::default()
            .components(components)
            .embed(embed)
            .ephemeral(true);
        msg.edit(*self, builder).await?;
        Ok(())
    }
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
    async fn get_player_from_discord_id(
        &self,
        discord_id: impl Into<Option<String>> + Clone,
    ) -> Result<Option<crate::database::models::Player>, BotError> {
        let id = match discord_id.into() {
            Some(id) => id,
            None => self.author().id.to_string(),
        };
        let player = self.data().database.get_player_by_discord_id(&id).await?;
        Ok(player)
    }

    async fn get_user_by_discord_id(
            &self,
            discord_id: impl Into<Option<String>> + Clone
        ) -> Result<Option<crate::database::models::User>, BotError> {
        let id = match discord_id.into() {
            Some(id) => id,
            None => self.author().id.to_string(),
        };
        let user = self.data().database.get_user_by_discord_id(&id).await?;
        Ok(user)
    }
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

    fn now(&self) -> poise::serenity_prelude::model::Timestamp {
        self.created_at()
    }

    async fn confirmation(
        &self,
        msg: &ReplyHandle<'_>,
        embed: CreateEmbed,
    ) -> Result<bool, BotError> {
        let reply = {
            let buttons = CreateActionRow::Buttons(vec![
                CreateButton::new("confirm")
                    .label("Confirm")
                    .style(ButtonStyle::Danger),
                CreateButton::new("cancel")
                    .label("Cancel")
                    .style(ButtonStyle::Primary),
            ]);
            CreateReply::default()
                .embed(embed)
                .components(vec![buttons])
        };
        msg.edit(*self, reply).await?;
        let mut ic = self.create_interaction_collector(msg).await?;
        while let Some(interactions) = &ic.next().await {
            match interactions.data.custom_id.as_str() {
                "confirm" => {
                    return Ok(true);
                }
                "cancel" => {
                    return Ok(false);
                }
                _ => {
                    continue;
                }
            }
        }
        Err(anyhow!("User did not respond in time"))
    }
}
