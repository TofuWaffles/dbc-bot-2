use std::str::FromStr;

use crate::utils::error::CommonError::*;
use crate::{
    database::{self},
    BotContext, BotError,
};
use anyhow::anyhow;
use poise::{
    serenity_prelude::{
        ChannelId, Colour, CreateEmbed, GuildChannel, GuildId, PartialGuild, Role, RoleId, User,
        UserId,
    },
    CreateReply, ReplyHandle,
};

pub trait DiscordTrait {
    #[inline]
    async fn to_user(ctx: &BotContext<'_>, id: &str) -> Result<User, BotError> {
        Ok(Self::to_user_id(id)?.to_user(ctx.http()).await?)
    }
    #[inline]
    fn to_user_id(id: impl Into<String>) -> Result<UserId, BotError> {
        Ok(UserId::from_str(&id.into())?)
    }

    async fn to_role(ctx: &BotContext<'_>, id: &str) -> Result<Role, BotError> {
        let roldid = RoleId::new(id.parse()?);
        let guild = ctx.guild().ok_or(NotInAGuild)?;
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
            .map_err(|_| GuildNotExists(id.to_string()).into())
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
