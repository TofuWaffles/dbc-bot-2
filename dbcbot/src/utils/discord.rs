use crate::{
    database::{self, models::Selectable},
    BotContext, BotError,
};
use anyhow::anyhow;
use futures::StreamExt;
use poise::{
    serenity_prelude::{
        self as serenity, Channel, ChannelId, ChannelType, Colour, ComponentInteractionCollector, ComponentInteractionDataKind::{ChannelSelect, RoleSelect}, CreateActionRow, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption, GuildChannel, GuildId, PartialGuild, Role, RoleId, User, UserId
    },
    CreateReply, ReplyHandle,
};
use crate::utils::error::CommonError::*;

pub trait DiscordTrait {
    async fn to_user(ctx: &BotContext<'_>, id: &str) -> Result<User, BotError> {
        Ok(UserId::new(id.parse()?).to_user(ctx.http()).await?)
    }

    async fn to_role(ctx: &BotContext<'_>, id: &str) -> Result<Role, BotError> {
        let roldid = RoleId::new(id.parse()?);
        let guild = ctx
            .guild()
            .ok_or(NotInAGuild)?;
        let role = guild.roles.get(&roldid).ok_or(anyhow!("Role not found"))?;
        Ok(role.clone())
    }

    async fn to_channel(ctx: &BotContext<'_>, id: &str) -> Result<GuildChannel, BotError> {
        ChannelId::new(id.parse()?)
            .to_channel(ctx.http())
            .await?
            .guild()
            .ok_or(anyhow!("Channel not found"))
    }

    async fn to_guild(ctx: &BotContext<'_>, id: &str) -> Result<PartialGuild, BotError> {
        GuildId::new(id.parse()?)
            .to_partial_guild(ctx.http())
            .await
            .map_err(|_|GuildNotExists(id.to_string()).into())
    }
}

pub trait UserExt {
    type Error;
    async fn full(
        &self,
        ctx: &BotContext<'_>,
    ) -> Result<Option<database::models::Player>, Self::Error>;
}

impl UserExt for User {
    type Error = BotError;
    async fn full(
        &self,
        ctx: &BotContext<'_>,
    ) -> Result<Option<database::models::Player>, Self::Error> {
        ctx.get_player_from_discord_id(self.id.to_string()).await
    }
}

use super::shorthand::BotContextExt;
pub async fn splash(ctx: &BotContext<'_>, msg: &ReplyHandle<'_>) -> Result<(), BotError> {
    let embed = CreateEmbed::default()
        .title("Loading next step...")
        .description("Please wait while we load the next step.")
        .colour(Colour::BLUE.0);

    let components = vec![];
    let builder = CreateReply::default()
        .components(components)
        .embed(embed)
        .ephemeral(true);

    msg.edit(*ctx, builder).await?;
    Ok(())
}

pub async fn select_channel(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    embed: CreateEmbed,
) -> Result<Channel, BotError>
{
    let component = vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
        "channel",
        CreateSelectMenuKind::Channel {
            default_channels: None,
            channel_types: Some(vec![ChannelType::Text]),
        },
    ))];
    let builder = CreateReply::default().embed(embed).components(component);
    msg.edit(*ctx, builder).await?;
    let mut ic = ctx.create_interaction_collector(msg).await?;
    while let Some(mci) = ic.next().await {
        mci.defer(ctx.http()).await?;
        if let ChannelSelect { values } = mci.data.kind {
            let channel = values[0].to_channel(ctx.http()).await?;
            return Ok(channel);
        }
    }
    Err(NoSelection.into())
}

pub async fn select_role(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    embed: CreateEmbed
) -> Result<Role, BotError>
{
    let component = vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
        "role",
        CreateSelectMenuKind::Role {
            default_roles: None,
        },
    ))];
    let builder = CreateReply::default().embed(embed).components(component);
    msg.edit(*ctx, builder).await?;
    while let Some(mci) = ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .filter(move |mci| mci.data.custom_id == "role")
        .await
    {
        mci.defer(ctx.http()).await?;
        if let RoleSelect { values } = mci.data.kind {
            let guild = ctx.guild().unwrap();
            let role = guild.roles.get(&values[0]).unwrap().clone();
            return Ok(role);
        }
    }
    Err(NoSelection.into())
}

pub async fn select_options<T: Selectable>(
    ctx: &BotContext<'_>,
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
    msg.edit(*ctx, builder).await?;
    let mut ic = ctx.create_interaction_collector(msg).await?;
    while let Some(interactions) = &ic.next().await {
        match interactions.data.custom_id.as_str() {
            "option" => {
                interactions.defer(ctx.http()).await?;
                if let poise::serenity_prelude::ComponentInteractionDataKind::StringSelect {
                    values,
                } = interactions.clone().data.kind
                {
                    return Ok(values[0].clone());
                }
            }
            button => {
                interactions.defer(ctx.http()).await?;
                return Ok(button.to_string());
            }
        }
    }
    Err(NoSelection.into())
}

pub async fn modal<T: poise::modal::Modal>(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    embed: CreateEmbed,
) -> Result<T, BotError> {
    let builder = {
        let components = vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new("open_modal")
                .label("Open modal")
                .style(poise::serenity_prelude::ButtonStyle::Success),
        ])];

        poise::CreateReply::default()
            .embed(embed)
            .components(components)
    };

    msg.edit(*ctx, builder).await?;

    if let Some(mci) = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
        .timeout(std::time::Duration::from_secs(120))
        .filter(move |mci| mci.data.custom_id == "open_modal")
        .await
    {
        let response = poise::execute_modal_on_component_interaction::<T>(
            ctx, mci, None, None,
        )
        .await?
        .ok_or(NoSelection)?;
        return Ok(response);
    }
    Err(NoSelection.into())
}
