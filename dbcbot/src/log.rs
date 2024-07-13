use std::{default, f32::consts::E, str::FromStr, time::SystemTime};

use anyhow::anyhow;
use poise::serenity_prelude::{ChannelId, Color, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage};
use strum::Display;
use tracing::info;

use crate::{database::Database, utils::shorthand::BotContextExt, BotContext, BotError};


pub enum State {
    SUCCESS = Color::DARK_GREEN.0 as isize,
    FAILURE = Color::RED.0 as isize,
    INFO = Color::BLUE.0 as isize,
    WARNING = Color::GOLD.0 as isize,
}
#[derive(Debug, Clone, Copy, Default, Display)]
pub enum Model{
    #[strum(to_string = "Player")]
    PLAYER,
    #[strum(to_string = "Database")]
    DATABASE,
    #[strum(to_string = "Brawl Stars API")]
    API,
    #[strum(to_string = "Marshal")]
    MARSHAL,
    #[strum(to_string = "Guild")]
    GUILD,
    #[strum(to_string = "Channel")]
    CHANNEL,
    #[strum(to_string = "Tournament")]
    TOURNAMENT,
    #[strum(to_string = "System")]
    SYSTEM,
    #[default]
    DEFAULT
}

pub struct Log {}

impl Log {
    async fn get_log_channel(ctx: BotContext<'_>) -> Result<ChannelId, BotError> {
        let guild_id = ctx
            .guild_id()
            .ok_or(anyhow!("Error getting log channel: Attempted to get log channel outside of a guild"))?
            .to_string();

        let log_channel = ChannelId::from_str(
            &ctx.data()
                .database
                .get_config(&guild_id)
                .await?
                .ok_or(anyhow!(
                    "Error getting log channel: config not found for guild {}",
                    guild_id
                ))?
                .log_channel_id,
        )?;

        Ok(log_channel)
    }
    fn author(ctx: &BotContext<'_>, model: Model) -> CreateEmbedAuthor{
        let (name, icon_url) = match model{
            Model::PLAYER | Model::MARSHAL => (ctx.author().name.clone(), ctx.author().avatar_url().unwrap_or_default()),
            Model::DATABASE => (model.to_string(), String::from("https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC93MmtCcnlHZk05eHdYbWtCZWpCaC5wbmcifQ:supercell:62YMWTV9LI8syf1HAJnKJTMkUEZR1-yXNqrxVHTHrB4?width=2400")),
            Model::API => (model.to_string(), String::from("https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9LWGU0ekxmSENqVlJTM2tmV0VzSy5wbmcifQ:supercell:SmOqSjpbIjqKqwrmZ2RWpEbwvBi1ERlMIp4Oe9fGI0g?width=2400")),
            Model::GUILD => (model.to_string(), Default::default()),
            Model::CHANNEL => (model.to_string(), Default::default()),
            Model::TOURNAMENT => (model.to_string(), Default::default()),
            Model::SYSTEM =>  (model.to_string(), Default::default()),
            Model::DEFAULT =>  (model.to_string(), Default::default()),
        };
        CreateEmbedAuthor::new(name).icon_url(icon_url)
    }

    fn thumbnail(state: State) -> String{
        match state{
            State::FAILURE => Default::default(),
            State::SUCCESS => Default::default(),
            State::INFO => Default::default(),
            State::WARNING => Default::default(),
        }
    }

    pub async fn log(
        ctx: BotContext<'_>,
        title: impl Into<String>,
        description: impl Into<String>,
        state: State,
        model: Model,
    ) -> Result<(), BotError>  
    {   
        let embed = CreateEmbed::default()
            .author(Self::author(&ctx, model))
            .title(title)
            .description(format!("**Action**\n{}", description.into()))
            .timestamp(ctx.now())
            .thumbnail(Self::thumbnail(state))
            .colour(State::SUCCESS as u32);
        let builder = CreateMessage::default().embed(embed);
        let channel = Self::get_log_channel(ctx).await?;
        channel.send_message(ctx, builder).await?;
        Ok(())
    }
}


struct Debug {}
/// Creates an info log message in the current guild's designated log channel.
pub async fn discord_log_info(
    ctx: BotContext<'_>,
    title: &str,
    mut fields: Vec<(&str, &str, bool)>,
) -> Result<(), BotError> {
    let guild_id = ctx
        .guild_id()
        .ok_or(anyhow!(
            "Error sending info log: Attempted to perform an info log outside of a guild"
        ))?
        .to_string();

    let log_channel = ChannelId::from_str(
        &ctx.data()
            .database
            .get_config(&guild_id)
            .await?
            .ok_or(anyhow!(
                "Error sending info log: config not found for guild {}",
                guild_id
            ))?
            .log_channel_id,
    )?;

    info!("ℹ️ {}\n\n{:#?}", title, fields);

    let now_string = format!(
        "<t:{}:F>",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );

    fields.push(("Happened at", &now_string, false));

    log_channel
        .send_message(
            ctx,
            CreateMessage::default().content("").embed(
                CreateEmbed::new()
                    .title(format!("ℹ️ {}", title))
                    .fields(fields)
                    .color(Color::BLURPLE),
            ),
        )
        .await?;

    Ok(())
}

/// Creates an error log message in the current guild's designated log channel.
pub async fn discord_log_error(
    ctx: BotContext<'_>,
    title: &str,
    mut fields: Vec<(&str, &str, bool)>,
) -> Result<(), BotError> {
    let guild_id = ctx
        .guild_id()
        .ok_or(anyhow!(
            "Error sending error log: Attempted to perform an info log outside of a guild"
        ))?
        .to_string();

    let log_channel = ChannelId::from_str(
        &ctx.data()
            .database
            .get_config(&guild_id)
            .await?
            .ok_or(anyhow!(
                "Error sending error log: config not found for guild {}",
                guild_id
            ))?
            .log_channel_id,
    )?;

    let now_string = format!(
        "<t:{}:F>",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    );

    fields.push(("Seen at", &now_string, false));

    log_channel
        .send_message(
            ctx,
            CreateMessage::default()
                .content("⚠️ An error occured in a command!")
                .embed(
                    CreateEmbed::new()
                        .title(title.to_string())
                        .description("Please check the logs for more information.")
                        .fields(fields)
                        .color(Color::RED),
                ),
        )
        .await?;

    Ok(())
}
