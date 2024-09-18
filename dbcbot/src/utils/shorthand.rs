use crate::{
    api::{
        brawlify::{BrawlMap, FullBrawler, FullGameMode, GameMode},
        official_brawl_stars::Brawler,
        APIResult,
    },
    database::{
        models::{GuildConfig, Mode},
        Database,
    },
    BotContext, BotError,
};
use anyhow::anyhow;
use futures::{Stream, StreamExt};
use crate::database::*;
use super::discord::select_options;
use poise::{
    serenity_prelude::{
        ButtonStyle, ComponentInteraction, CreateActionRow, CreateButton, CreateEmbed,
        CreateEmbedFooter,
    },
    CreateReply, ReplyHandle,
};
use tokio::time::Duration;
pub trait BotContextExt<'a> {
    async fn mode_selection(&self, msg: &ReplyHandle<'_>) -> Result<FullGameMode, BotError>;
    async fn map_selection(&self, msg: &ReplyHandle<'_>, mode: &Mode)
        -> Result<BrawlMap, BotError>;
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
        discord_id: impl Into<Option<String>> + Clone,
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

    async fn dismiss(&self, msg: &ReplyHandle<'_>) -> Result<(), BotError>;

    /// Prompt the user to select a brawler.
    /// # Arguments
    /// * `ctx` - The context of the command.
    /// * `msg` - The message to reply to.
    /// # Returns
    /// Returns a `Result` containing a `String` representing the selected brawler.
    async fn brawler_selection(&self, msg: &ReplyHandle<'_>) -> Result<Brawler, BotError>;
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
        let components = match buttons.into() {
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
        discord_id: impl Into<Option<String>> + Clone,
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
                    interactions.defer(self.http()).await?;
                    return Ok(true);
                }
                "cancel" => {
                    interactions.defer(self.http()).await?;
                    return Ok(false);
                }
                _ => {
                    continue;
                }
            }
        }
        Err(anyhow!("User did not respond in time"))
    }

    async fn dismiss(&self, msg: &ReplyHandle<'_>) -> Result<(), BotError> {
        self.prompt(
            msg,
            CreateEmbed::new().description("You can dismiss this safely!"),
            None,
        )
        .await
    }

    async fn brawler_selection(&self, msg: &ReplyHandle<'_>) -> Result<Brawler, BotError> {
        const CAPACITY: usize = 25;
        let brawlers = match self.data().apis.brawlify.get_brawlers().await? {
            APIResult::Ok(b) => b,
            APIResult::NotFound => return Err(anyhow!("Brawlers not found")),
            APIResult::Maintenance => {
                return Err(anyhow!("Brawlify is currently undergoing maintenance"))
            }
        };
        let reply = {
            let embed = CreateEmbed::default().description("Select how you would like to sort the brawler list.\n 🅰️: Sort in alphabetical order.\n💎: Sort by rarity");
            let buttons = CreateActionRow::Buttons(vec![
                CreateButton::new("0")
                    .label("🅰️")
                    .style(ButtonStyle::Primary),
                CreateButton::new("1")
                    .label("💎")
                    .style(ButtonStyle::Primary),
            ]);
            CreateReply::default()
                .embed(embed)
                .components(vec![buttons])
        };
        msg.edit(*self, reply).await?;
        let (prev, next) = (String::from("prev"), String::from("next"));
        let buttons = CreateActionRow::Buttons(vec![
            CreateButton::new(prev.clone())
                .label("⬅️")
                .style(ButtonStyle::Primary),
            CreateButton::new(next.clone())
                .label("➡️")
                .style(ButtonStyle::Primary),
        ]);
        let mut page_number: usize = 0;
        let embed = |brawlers: &[FullBrawler]| {
            CreateEmbed::default()
            .title("Brawler Selection")
            .description(format!(
                "Select a brawler to view more information about them. Use the buttons below to navigate through the list."
            ))
            .fields(brawlers.iter().map(|b|{
                (b.name.clone(),"",true)
            }).collect::<Vec<(String,&str,bool)>>())
        };
        let mut ic = self.create_interaction_collector(msg).await?;

        while let Some(interactions) = &ic.next().await {
            interactions.defer(self.http()).await?;
            match interactions.data.custom_id.as_str() {
                "0" => {
                    let brawlers: Vec<Vec<FullBrawler>> = brawlers
                        .sort_by_alphabet()
                        .chunks(CAPACITY)
                        .map(|chunk| chunk.to_vec())
                        .collect();
                    let mut chunk: &[FullBrawler] = brawlers[page_number].as_ref();
                    loop {
                        let selected =
                            select_options(self, msg, embed(chunk), vec![buttons.clone()], chunk)
                                .await?;
                        match selected.as_str() {
                            "prev" => {
                                page_number = page_number.checked_sub(1).unwrap_or(0);
                            }
                            "next" => {
                                page_number = (page_number + 1).min(brawlers.len() - 1);
                            }
                            identifier @ _ => {
                                println!("Selected brawler id: {}", identifier);
                                let brawler = chunk
                                    .iter()
                                    .find(|b| (**b).id == identifier.parse::<i32>().unwrap())
                                    .unwrap()
                                    .to_owned();
                                return Ok(brawler.into());
                            }
                        }

                        chunk = brawlers[page_number].as_ref();
                        println!("Updated page number: {}", page_number);
                    }
                }
                "1" => {}
                _ => unreachable!(),
            }
        }
        Err(anyhow!("User did not respond in time"))
    }

    async fn map_selection(
        &self,
        msg: &ReplyHandle<'_>,
        mode: &Mode,
    ) -> Result<BrawlMap, BotError> {
        let maps = match self.data().apis.brawlify.get_maps().await? {
            APIResult::Ok(m) => m,
            APIResult::NotFound => return Err(anyhow!("Maps not found")),
            APIResult::Maintenance => {
                return Err(anyhow!("Brawlify is currently undergoing maintenance"))
            }
        };
        let filtered_maps = maps.filter_map_by_mode(mode);
        let (prev, select, any, next) = (
            String::from("prev"),
            String::from("any"),
            String::from("select"),
            String::from("next"),
        );
        let buttons = CreateActionRow::Buttons(vec![
            CreateButton::new(prev.clone())
                .label("⬅️")
                .style(ButtonStyle::Primary),
            CreateButton::new(select.clone())
                .label("Select")
                .style(ButtonStyle::Primary),
            CreateButton::new(any.clone())
                .label("Any")
                .style(ButtonStyle::Primary),
            CreateButton::new(next.clone())
                .label("➡️")
                .style(ButtonStyle::Primary),
        ]);
        let reply = |map: BrawlMap| {
            let embed = CreateEmbed::default()
                .title(format!("{}", map.name))
                .description(format!(
                    "Environment: ** {}**\nMode: **{}**\nAvailability: **{}**",
                    map.environment.name,
                    map.game_mode.name,
                    ["Yes", "No"][(!map.disabled) as usize]
                ))
                .image(map.image_url)
                .thumbnail(map.game_mode.image_url)
                .footer(CreateEmbedFooter::new("Provided by Brawlify"));
            CreateReply::default()
                .embed(embed)
                .components(vec![buttons.clone()])
        };
        let mut page_number: usize = 0;
        msg.edit(*self, reply(filtered_maps[page_number].to_owned()))
            .await?;
        let mut ic = self.create_interaction_collector(msg).await?;
        while let Some(interactions) = &ic.next().await {
            match interactions.data.custom_id.as_str() {
                "prev" => {
                    page_number = page_number.checked_sub(1).unwrap_or(0);
                }
                "select" => {
                    interactions.defer(self.http()).await?;
                    return Ok(filtered_maps[page_number].clone());
                }
                "any" => {
                    interactions.defer(self.http()).await?;
                    return Ok(BrawlMap::default());
                }
                "next" => {
                    page_number = (page_number + 1).min(filtered_maps.len() - 1);
                }
                _ => unreachable!(),
            }
            interactions.defer(self.http()).await?;
            msg.edit(*self, reply(filtered_maps[page_number].to_owned()))
                .await?;
        }
        Err(anyhow!("User did not respond in time"))
    }

    async fn mode_selection(&self, msg: &ReplyHandle<'_>) -> Result<FullGameMode, BotError> {
        let modes = match self.data().apis.brawlify.get_modes().await? {
            APIResult::Ok(m) => m,
            APIResult::NotFound => return Err(anyhow!("Modes not found")),
            APIResult::Maintenance => {
                return Err(anyhow!("Brawlify is currently undergoing maintenance"))
            }
        };
        let (prev, select, next) = (
            String::from("prev"),
            String::from("select"),
            String::from("next"),
        );
        let buttons = CreateActionRow::Buttons(vec![
            CreateButton::new(prev.clone())
                .label("⬅️")
                .style(ButtonStyle::Primary),
            CreateButton::new(select.clone())
                .label("Select")
                .style(ButtonStyle::Primary),
            CreateButton::new(next.clone())
                .label("➡️")
                .style(ButtonStyle::Primary),
        ]);
        let reply = |mode: FullGameMode| {
            let embed = CreateEmbed::default()
                .title(format!("{}", mode.name))
                .description(format!(
                    "Description: **{}**\nAvailability: **{}**",
                    mode.description,
                    ["Yes", "No"][(mode.disabled) as usize]
                ))
                .thumbnail(mode.image_url)
                .footer(CreateEmbedFooter::new("Provided by Brawlify"));
            CreateReply::default()
                .embed(embed)
                .components(vec![buttons.clone()])
        };
        let mut page_number: usize = 0;
        let mut selected = modes.list[page_number].to_owned();
        msg.edit(*self, reply(selected.to_owned())).await?;
        let mut ic = self.create_interaction_collector(msg).await?;
        while let Some(interactions) = &ic.next().await {
            match interactions.data.custom_id.as_str() {
                "prev" => {
                    page_number = page_number.checked_sub(1).unwrap_or(0);
                }
                "select" => {
                    interactions.defer(self.http()).await?;
                    return Ok(selected.clone());
                }
                "next" => {
                    page_number = (page_number + 1).min(modes.list.len() - 1);
                }
                _ => unreachable!(),
            }
            interactions.defer(self.http()).await?;
            selected = modes.list[page_number].to_owned();
            msg.edit(*self, reply(selected.to_owned())).await?;
        }
        Err(anyhow!("User did not respond in time"))
    }
}
