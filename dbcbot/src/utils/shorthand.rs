use super::error::CommonError;
use crate::database::*;
use crate::utils::error::CommonError::*;
use crate::{
    api::{
        brawlify::{BrawlMap, FullBrawler, FullGameMode},
        official_brawl_stars::Brawler,
        APIResult,
    },
    database::models::{GuildConfig, Mode},
    BotContext, BotError,
};
use anyhow::anyhow;
use futures::{Stream, StreamExt};
use models::Selectable;
use poise::serenity_prelude::{CacheHttp, CreateInteractionResponse};
use poise::{
    serenity_prelude::{
        self as serenity, ButtonStyle, Channel, ChannelType, ComponentInteraction,
        ComponentInteractionCollector,
        ComponentInteractionDataKind::{ChannelSelect, RoleSelect},
        CreateActionRow, CreateButton, CreateEmbed, CreateEmbedFooter, CreateSelectMenu,
        CreateSelectMenuKind, CreateSelectMenuOption, Role, UserId,
    },
    CreateReply, ReplyHandle,
};
use std::str::FromStr;

use tokio::time::Duration;
pub trait BotContextExt<'a> {
    async fn default_map(&self, tournament_id: i32) -> Result<(), BotError>;
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

    async fn get_current_round(&self, tournament_id: i32) -> Result<i32, BotError>;

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

    async fn get_config(&self) -> Result<Option<GuildConfig>, BotError> {
        let guild_id = self.guild_id().ok_or(NotInAGuild)?;
        let config = self.data().database.get_config(&guild_id).await?;
        Ok(config)
    }

    async fn default_map(&self, tournament_id: i32) -> Result<(), BotError> {
        self.data().database.set_default_map(tournament_id).await
    }

    async fn get_player_from_discord_id(
        &self,
        discord_id: impl Into<Option<String>> + Clone,
    ) -> Result<Option<crate::database::models::Player>, BotError> {
        let id = match discord_id.into() {
            Some(id) => UserId::from_str(&id).unwrap(),
            None => self.author().id,
        };
        let player = self.data().database.get_player_by_discord_id(&id).await?;
        Ok(player)
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
        let guild_id = self.guild_id().ok_or(NotInAGuild)?;
        let round = self
            .data()
            .database
            .get_tournament(&guild_id, tournament_id)
            .await?
            .map_or_else(|| 0, |t| t.current_round);
        Ok(round)
    }

    fn now(&self) -> poise::serenity_prelude::model::Timestamp {
        self.created_at()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Component<'a> {
    pub ctx: BotContext<'a>,
}

pub trait BotComponent<'a> {
    fn components(&self) -> Component;
}

impl<'a> BotComponent<'a> for BotContext<'a> {
    fn components(&self) -> Component {
        Component { ctx: *self }
    }
}

impl<'a> Component<'a> {
    const USER_CMD_TIMEOUT: u64 = 120_000;
    async fn create_interaction_collector(
        &self,
        msg: &ReplyHandle<'_>,
    ) -> Result<impl Stream<Item = ComponentInteraction> + Unpin, BotError> {
        let ic = msg
            .clone()
            .into_message()
            .await?
            .await_component_interaction(&self.ctx.serenity_context().shard)
            .timeout(Duration::from_millis(Self::USER_CMD_TIMEOUT))
            .stream();
        Ok(ic)
    }

    pub async fn confirmation(
        &self,
        msg: &ReplyHandle<'_>,
        embed: CreateEmbed,
    ) -> Result<bool, BotError> {
        let reply = {
            let buttons = CreateActionRow::Buttons(vec![
                CreateButton::new("cancel")
                    .label("Cancel")
                    .style(ButtonStyle::Danger),
                CreateButton::new("confirm")
                    .label("Confirm")
                    .style(ButtonStyle::Success),
            ]);
            CreateReply::default()
                .embed(embed)
                .components(vec![buttons])
        };
        msg.edit(self.ctx, reply).await?;
        let mut ic = self.create_interaction_collector(msg).await?;
        while let Some(interactions) = &ic.next().await {
            interactions.acknowledge(&self.ctx).await?;
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

    pub async fn prompt(
        &self,
        msg: &ReplyHandle<'a>,
        embed: CreateEmbed,
        buttons: impl Into<Option<Vec<CreateButton>>>,
    ) -> Result<(), BotError> {
        let components = match buttons.into() {
            Some(buttons) => {
                let chunked_buttons: Vec<Vec<CreateButton>> =
                    buttons.chunks(5).map(|c| c.to_vec()).collect();
                chunked_buttons
                    .iter()
                    .map(|chunk| CreateActionRow::Buttons(chunk.to_vec()))
                    .collect()
            }
            _ => vec![],
        };
        let builder = CreateReply::default()
            .components(components)
            .embed(embed)
            .ephemeral(true);
        msg.edit(self.ctx, builder).await?;
        Ok(())
    }

    pub async fn dismiss(&self, msg: &ReplyHandle<'_>) -> Result<(), BotError> {
        self.prompt(
            msg,
            CreateEmbed::new().description("You can dismiss this safely!"),
            None,
        )
        .await
    }

    pub async fn brawler_selection(&self, msg: &ReplyHandle<'_>) -> Result<Brawler, BotError> {
        const CAPACITY: usize = 25;
        let brawlers = match self.ctx.data().apis.brawlify.get_brawlers().await? {
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
        msg.edit(self.ctx, reply).await?;
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
            .description("Select a brawler to view more information about them. Use the buttons below to navigate through the list.".to_string())
            .fields(brawlers.iter().map(|b|{
                (b.name.clone(),"",true)
            }).collect::<Vec<(String,&str,bool)>>())
        };
        let mut ic = self.create_interaction_collector(msg).await?;

        while let Some(interactions) = &ic.next().await {
            interactions.acknowledge(&self.ctx).await?;
            match interactions.data.custom_id.as_str() {
                "0" => {
                    let brawlers: Vec<Vec<FullBrawler>> = brawlers
                        .clone()
                        .sort_by_alphabet()
                        .chunks(CAPACITY)
                        .map(|chunk| chunk.to_vec())
                        .collect();
                    let mut chunk: &[FullBrawler] = brawlers[page_number].as_ref();
                    loop {
                        let selected = match self
                            .select_options(msg, embed(chunk), vec![buttons.clone()], chunk)
                            .await
                        {
                            Ok(s) => s,
                            Err(_) => break,
                        };

                        match selected.as_str() {
                            "prev" => {
                                page_number = page_number.saturating_sub(1);
                            }
                            "next" => {
                                page_number = (page_number + 1).min(brawlers.len() - 1);
                            }
                            identifier => {
                                let brawler = chunk
                                    .iter()
                                    .find(|b| b.id == identifier.parse::<i32>().unwrap())
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
        Err(anyhow!("No option selected"))
    }

    pub async fn map_selection(
        &self,
        msg: &ReplyHandle<'_>,
        mode: &Mode,
    ) -> Result<BrawlMap, BotError> {
        if mode.is_unknown() {
            self.prompt(
                msg,
                CreateEmbed::new().description("No maps detected for Unknown mode"),
                None,
            )
            .await?;
            return Err(NoSelection.into());
        }
        let raw = self.ctx.data().apis.brawlify.get_maps().await?;
        let maps = raw
            .handler(&self.ctx, msg)
            .await?
            .ok_or(CommonError::APIError("No maps found".to_string()))?;
        let filtered_maps = maps.to_owned().filter_map_by_mode(mode);
        let (prev, select, any, next) = (
            String::from("prev"),
            String::from("select"),
            String::from("any"),
            String::from("next"),
        );
        let reply = |map: BrawlMap| {
            let embed = CreateEmbed::default()
                .title(map.name.to_string())
                .fields(vec![
                    ("ID", map.id.to_string(), true),
                    ("Name", map.name, true),
                    ("Environment", map.environment.name, true),
                    ("Mode", map.game_mode.name, true),
                    (
                        "Availability",
                        ["Yes", "No"][(map.disabled) as usize].to_string(),
                        true,
                    ),
                ])
                .image(map.image_url)
                .thumbnail(map.game_mode.image_url)
                .footer(CreateEmbedFooter::new("Provided by Brawlify"));
            let buttons = CreateActionRow::Buttons(vec![
                CreateButton::new(prev.clone())
                    .label("⬅️")
                    .style(ButtonStyle::Primary),
                CreateButton::new(select.clone())
                    .label("Select")
                    .style(ButtonStyle::Primary)
                    .disabled(map.disabled),
                CreateButton::new(any.clone())
                    .label("Any")
                    .style(ButtonStyle::Primary),
                CreateButton::new(next.clone())
                    .label("➡️")
                    .style(ButtonStyle::Primary),
            ]);
            CreateReply::default()
                .embed(embed)
                .components(vec![buttons])
        };
        let mut page_number: usize = 0;
        msg.edit(self.ctx, reply(filtered_maps[page_number].to_owned()))
            .await?;
        let mut ic = self.create_interaction_collector(msg).await?;
        while let Some(interactions) = &ic.next().await {
            match interactions.data.custom_id.as_str() {
                "prev" => {
                    page_number = page_number.saturating_sub(1);
                }
                "select" => {
                    interactions.acknowledge(&self.ctx).await?;
                    return Ok(filtered_maps[page_number].clone());
                }
                "any" => {
                    interactions.acknowledge(&self.ctx).await?;
                    let selected = BrawlMap::default();
                    return Ok(selected);
                }
                "next" => {
                    page_number = (page_number + 1).min(filtered_maps.len() - 1);
                }
                _ => unreachable!(),
            }
            interactions.acknowledge(&self.ctx).await?;
            msg.edit(self.ctx, reply(filtered_maps[page_number].to_owned()))
                .await?;
        }
        Err(NoSelection.into())
    }

    pub async fn mode_selection(&self, msg: &ReplyHandle<'_>) -> Result<FullGameMode, BotError> {
        let raw = self.ctx.data().apis.brawlify.get_modes().await?;
        let modes = raw
            .handler(&self.ctx, msg)
            .await?
            .ok_or(CommonError::APIError("No modes found".to_string()))?;
        let (prev, select, next) = (
            String::from("prev"),
            String::from("select"),
            String::from("next"),
        );

        let reply = |mode: FullGameMode| {
            let embed = CreateEmbed::default()
                .title(mode.name.to_string())
                .fields(vec![
                    ("ID", mode.sc_id.to_string(), true),
                    ("Name", mode.name.clone(), true),
                    (
                        "Availability",
                        String::from(["Yes", "No"][(mode.disabled) as usize]),
                        true,
                    ),
                    ("Code name", mode.sc_hash.clone(), true),
                ])
                .description(format!("Description: {}", mode.description))
                .thumbnail(mode.image_url)
                .footer(CreateEmbedFooter::new("Provided by Brawlify"));
            let buttons = CreateActionRow::Buttons(vec![
                CreateButton::new(prev.clone())
                    .label("⬅️")
                    .style(ButtonStyle::Primary),
                CreateButton::new(select.clone())
                    .label("Select")
                    .style(ButtonStyle::Primary)
                    .disabled(mode.disabled),
                CreateButton::new(next.clone())
                    .label("➡️")
                    .style(ButtonStyle::Primary),
            ]);
            CreateReply::default()
                .embed(embed)
                .components(vec![buttons])
        };
        let mut page_number: usize = 0;
        let mut selected = modes.list[page_number].to_owned();
        msg.edit(self.ctx, reply(selected.to_owned())).await?;
        let mut ic = self.create_interaction_collector(msg).await?;
        while let Some(interactions) = &ic.next().await {
            match interactions.data.custom_id.as_str() {
                "prev" => {
                    page_number = page_number.saturating_sub(1);
                }
                "select" => {
                    interactions.acknowledge(&self.ctx).await?;
                    return Ok(selected.clone());
                }
                "next" => {
                    page_number = (page_number + 1).min(modes.list.len() - 1);
                }
                _ => unreachable!(),
            }
            interactions.acknowledge(&self.ctx).await?;
            selected = modes.list[page_number].to_owned();
            msg.edit(self.ctx, reply(selected.to_owned())).await?;
        }
        Err(NoSelection.into())
    }

    pub async fn select_channel(
        &self,
        msg: &ReplyHandle<'_>,
        embed: CreateEmbed,
    ) -> Result<Channel, BotError> {
        let component = vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
            "channel",
            CreateSelectMenuKind::Channel {
                default_channels: None,
                channel_types: Some(vec![ChannelType::Text]),
            },
        ))];
        let builder = CreateReply::default().embed(embed).components(component);
        msg.edit(self.ctx, builder).await?;
        let mut ic = self.create_interaction_collector(msg).await?;
        while let Some(mci) = ic.next().await {
            mci.acknowledge(&self.ctx).await?;
            if let ChannelSelect { values } = mci.data.kind {
                let channel = values[0].to_channel(self.ctx.http()).await?;
                return Ok(channel);
            }
        }
        Err(NoSelection.into())
    }

    pub async fn select_role(
        &self,
        msg: &ReplyHandle<'_>,
        embed: CreateEmbed,
    ) -> Result<Role, BotError> {
        let component = vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
            "role",
            CreateSelectMenuKind::Role {
                default_roles: None,
            },
        ))];
        let builder = CreateReply::default().embed(embed).components(component);
        msg.edit(self.ctx, builder).await?;
        if let Some(mci) = ComponentInteractionCollector::new(self.ctx)
            .author_id(self.ctx.author().id)
            .channel_id(self.ctx.channel_id())
            .timeout(std::time::Duration::from_secs(120))
            .filter(move |mci| mci.data.custom_id == "role")
            .await
        {
            mci.defer(self.ctx.http()).await?;
            if let RoleSelect { values } = mci.data.kind {
                let guild = self.ctx.guild().unwrap();
                let role = guild.roles.get(&values[0]).unwrap().clone();
                return Ok(role);
            }
        }
        Err(NoSelection.into())
    }

    pub async fn select_options<T: Selectable>(
        &self,
        msg: &ReplyHandle<'_>,
        embed: CreateEmbed,
        buttons: impl Into<Option<Vec<CreateActionRow>>>,
        items: &[T],
    ) -> Result<String, BotError> {
        let options = items
            .iter()
            .map(|t| CreateSelectMenuOption::new(t.label(), t.identifier()))
            .collect();
        let mut buttons: Vec<CreateActionRow> = buttons.into().unwrap_or(vec![]);
        let mut component = vec![CreateActionRow::SelectMenu(
            CreateSelectMenu::new("option", CreateSelectMenuKind::String { options })
                .disabled(items.is_empty()),
        )];
        component.append(&mut buttons);

        let builder = CreateReply::default().embed(embed).components(component);
        msg.edit(self.ctx, builder).await?;
        let mut ic = self.create_interaction_collector(msg).await?;
        if let Some(interactions) = &ic.next().await {
            match interactions.data.custom_id.as_str() {
                "option" => {
                    interactions.defer(self.ctx.http()).await?;
                    if let poise::serenity_prelude::ComponentInteractionDataKind::StringSelect {
                        values,
                    } = interactions.clone().data.kind
                    {
                        return Ok(values[0].clone());
                    }
                }
                button => {
                    interactions.defer(self.ctx.http()).await?;
                    return Ok(button.to_string());
                }
            }
        }
        Err(NoSelection.into())
    }

    pub async fn modal<T: poise::modal::Modal>(
        &self,
        msg: &ReplyHandle<'_>,
        embed: CreateEmbed,
    ) -> Result<T, BotError> {
        let builder = {
            let components = vec![serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("open_modal")
                    .label("Continue")
                    .style(poise::serenity_prelude::ButtonStyle::Success),
            ])];

            poise::CreateReply::default()
                .embed(embed)
                .components(components)
        };

        msg.edit(self.ctx, builder).await?;

        if let Some(mci) = serenity::ComponentInteractionCollector::new(self.ctx.serenity_context())
            .timeout(std::time::Duration::from_secs(120))
            .filter(move |mci| mci.data.custom_id == "open_modal")
            .await
        {
            let response =
                poise::execute_modal_on_component_interaction::<T>(self.ctx, mci, None, None)
                    .await?
                    .ok_or(NoSelection)?;
            return Ok(response);
        }
        Err(NoSelection.into())
    }

    pub async fn get_msg(&self) -> Result<ReplyHandle, BotError> {
        let embed = CreateEmbed::default()
            .title("Loading command...")
            .description("Please wait while we load the this command.");
        let reply = CreateReply::default().embed(embed).ephemeral(true);
        let msg = self.ctx.send(reply).await?;
        Ok(msg)
    }
}

pub trait ComponentInteractionExt {
    /// Shorthand for Acknowledge the interaction
    async fn acknowledge(&self, ctx: impl CacheHttp) -> Result<(), BotError>;
}

impl ComponentInteractionExt for ComponentInteraction {
    async fn acknowledge(&self, cache: impl CacheHttp) -> Result<(), BotError> {
        Ok(self
            .create_response(cache, CreateInteractionResponse::Acknowledge)
            .await?)
    }
}
