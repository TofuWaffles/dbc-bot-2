use std::{str::FromStr, time::SystemTime};

use anyhow::anyhow;
use poise::serenity_prelude::{ChannelId, Color, CreateEmbed, CreateMessage};
use tracing::info;

use crate::{database::Database, BotContext, BotError};

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
                        .title(format!("{}", title))
                        .description("Please check the logs for more information.")
                        .fields(fields)
                        .color(Color::RED),
                ),
        )
        .await?;

    Ok(())
}
