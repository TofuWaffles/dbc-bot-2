use crate::{database::models::Selectable, BotContext, BotError};
use anyhow::anyhow;
use poise::{
    serenity_prelude::{
        self as serenity, Channel, ChannelType, Colour, ComponentInteractionCollector,
        ComponentInteractionDataKind::{ChannelSelect, RoleSelect},
        CreateActionRow, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind,
        CreateSelectMenuOption, Role,
    },
    CreateReply, ReplyHandle,
};

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

pub async fn select_channel<S>(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    title: S,
    description: S,
) -> Result<Channel, BotError>
where
    S: Into<String> + Send + 'static,
{
    let embed = CreateEmbed::default()
        .title(title)
        .description(description)
        .color(Colour::GOLD);
    let component = vec![CreateActionRow::SelectMenu(CreateSelectMenu::new(
        "channel",
        CreateSelectMenuKind::Channel {
            default_channels: None,
            channel_types: Some(vec![ChannelType::Text]),
        },
    ))];
    let builder = CreateReply::default().embed(embed).components(component);
    msg.edit(*ctx, builder).await?;
    while let Some(mci) = ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .filter(move |mci| mci.data.custom_id == "channel")
        .await
    {
        mci.defer(ctx.http()).await?;
        if let ChannelSelect { values } = mci.data.kind {
            let channel = values[0].to_channel(ctx.http()).await?;
            return Ok(channel);
        }
    }
    Err(anyhow!("No channel selected"))
}

pub async fn select_role<S>(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    title: S,
    description: S,
) -> Result<Role, BotError>
where
    S: Into<String> + Send + 'static,
{
    let embed = CreateEmbed::default()
        .title(title.into())
        .description(description.into())
        .color(Colour::GOLD);
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
    Err(anyhow!("No role selected"))
}

pub async fn select_options<T: Selectable>(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    title: impl Into<String> + Send + 'static,
    description: impl Into<String> + Send + 'static,
    items: &[T],
) -> Result<String, BotError> {
    let embed = CreateEmbed::default()
        .title(title.into())
        .description(description.into())
        .color(Colour::GOLD);

    let options = items
        .iter()
        .map(|t| CreateSelectMenuOption::new(t.label(), t.identifier()))
        .collect();
    let component = vec![CreateActionRow::SelectMenu(
        CreateSelectMenu::new("option", CreateSelectMenuKind::String { options })
            .disabled(items.is_empty()),
    )];
    let builder = CreateReply::default().embed(embed).components(component);
    msg.edit(*ctx, builder).await?;
    while let Some(mci) = ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .filter(move |mci| mci.data.custom_id == "option")
        .await
    {
        mci.defer(ctx.http()).await?;
        if let poise::serenity_prelude::ComponentInteractionDataKind::StringSelect { values } =
            mci.data.kind
        {
            return Ok(values[0].clone());
        }
    }
    Err(anyhow!("No option selected"))
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
        .ok_or(anyhow!("Modal interaction from <@{}> returned None. This may mean that the modal has timed out.", ctx.author().id.to_string()))?;
        return Ok(response);
    }
    Err(anyhow!("No name entered"))
}
