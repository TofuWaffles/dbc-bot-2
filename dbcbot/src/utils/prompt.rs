use crate::{BotContext, BotError};
use poise::{
    serenity_prelude::{Colour, CreateActionRow, CreateEmbed},
    CreateReply, ReplyHandle,
};

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