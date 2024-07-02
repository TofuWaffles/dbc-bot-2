use std::{str::FromStr, time::SystemTime};

use poise::{
    serenity_prelude::{ChannelId, Color, CreateEmbed, CreateMessage},
    CreateReply,
};

use crate::{
    database::{models::Tournament, Database},
    BotError, Context,
};

pub async fn discord_log_info(ctx: Context<'_>, msg: CreateMessage) -> Result<(), BotError> {
    let guild_id = ctx
        .guild_id()
        .ok_or("Error sending info log: Attempted to perform an info log outside of a guild")?
        .to_string();

    let log_channel = ChannelId::from_str(
        &ctx.data()
            .database
            .get_config(&guild_id)
            .await?
            .ok_or(format!(
                "Error sending info log: config not found for guild {}",
                guild_id
            ))?
            .log_channel_id,
    )?;

    log_channel.send_message(ctx, msg).await?;

    Ok(())
}

pub async fn discord_log_error(
    ctx: Context<'_>,
    title: &str,
    user: Option<crate::database::models::User>,
    tournament: Vec<Tournament>,
) -> Result<(), BotError> {
    let guild_id = ctx
        .guild_id()
        .ok_or("Error sending error log: Attempted to perform an info log outside of a guild")?
        .to_string();

    let log_channel = ChannelId::from_str(
        &ctx.data()
            .database
            .get_config(&guild_id)
            .await?
            .ok_or(format!(
                "Error sending error log: config not found for guild {}",
                guild_id
            ))?
            .log_channel_id,
    )?;

    log_channel
        .send_message(
            ctx,
            CreateMessage::default()
                .content("⚠️ An error occured in a command!")
                .embed(
                    CreateEmbed::new()
                        .title(format!("{}", title))
                        .description("Please check the logs for more information.")
                        .fields(vec![
                            ("User", &ctx.author().name, false),
                            (
                                "Seen at",
                                &format!(
                                    "<t:{}:F>",
                                    SystemTime::now()
                                        .duration_since(SystemTime::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs()
                                ),
                                false,
                            ),
                            ("Player ID", &format!("{:#?}", user), false),
                            ("Tournament", &format!("{:#?}", tournament), false),
                        ])
                        .color(Color::RED),
                ),
        )
        .await?;

    Ok(())
}
