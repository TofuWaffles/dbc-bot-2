use crate::{BotContext, BotError};
use poise::{
    serenity_prelude::{Channel, ChannelType, Colour, ComponentInteractionCollector, CreateActionRow, CreateEmbed, CreateSelectMenu, CreateSelectMenuKind, Role, ComponentInteractionDataKind::{ChannelSelect, RoleSelect}},
    CreateReply, ReplyHandle,
};
use anyhow::anyhow;

pub async fn prompt<S,O>(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    title: S,
    description: S,
    color: impl Into<Option<u32>>
) -> Result<(), BotError>
where
    S: Into<String> + Send + 'static,
{
    let embed = CreateEmbed::default()
        .title(title.into())
        .description(description.into())
        .colour(color.into().unwrap_or(Colour::BLUE.0));

    let components = vec![];
    let builder = CreateReply::default()
        .components(components)
        .embed(embed)
        .ephemeral(true);

    msg.edit(*ctx, builder).await?;

    Ok(())
}

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
        match mci.data.kind {
            ChannelSelect { values } => {
                let channel = values[0].to_channel(ctx.http()).await?;
                return Ok(channel);
            }
            _ => {}
        }
    }
    Err(anyhow!("No channel selected").into())
}

pub async fn select_role<S>(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    title: S,
    description: S,
) -> Result<Role, BotError>where
S: Into<String> + Send + 'static {
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
        match mci.data.kind {
            RoleSelect { values } => {
                let role = ctx.guild().unwrap().roles.get(&values[0]).unwrap().clone();
                return Ok(role);
            }
            _ => {}
        }
    }
    Err(anyhow!("No role selected").into())
}
