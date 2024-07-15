use std::{str::FromStr, time::SystemTime};

use anyhow::anyhow;
use poise::serenity_prelude::{ChannelId, Color, CreateEmbed, CreateEmbedAuthor, CreateMessage};
use strum::Display;

use crate::{database::Database, utils::shorthand::BotContextExt, BotContext, BotError};

pub enum State {
    SUCCESS = Color::DARK_GREEN.0 as isize,
    FAILURE = Color::RED.0 as isize,
    INFO = Color::BLUE.0 as isize,
    WARNING = Color::GOLD.0 as isize,
}
#[derive(Debug, Clone, Copy, Default, Display)]
pub enum Model {
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
    DEFAULT,
}

pub trait Log {
    async fn get_log_channel(&self) -> Result<ChannelId, BotError>;
    fn get_author_img(&self, model: &Model) -> CreateEmbedAuthor;
    fn thumbnail(&self, state: &State) -> String;
    async fn log(
        &self,
        title: impl Into<String>,
        description: impl Into<String>,
        state: State,
        model: Model,
    ) -> Result<(), BotError>;
}
impl Log for BotContext<'_> {
    async fn get_log_channel(&self) -> Result<ChannelId, BotError> {
        let guild_id = self
            .guild_id()
            .ok_or(anyhow!(
                "Error getting log channel: Attempted to get log channel outside of a guild"
            ))?
            .to_string();

        let log_channel = ChannelId::from_str(
            &self
                .data()
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
    fn get_author_img(&self, model: &Model) -> CreateEmbedAuthor {
        let (name, icon_url) = match model{
            Model::PLAYER | Model::MARSHAL => (self.author().name.clone(), self.author().avatar_url().unwrap_or_default()),
            Model::DATABASE => (model.to_string(), String::from("https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC93MmtCcnlHZk05eHdYbWtCZWpCaC5wbmcifQ:supercell:62YMWTV9LI8syf1HAJnKJTMkUEZR1-yXNqrxVHTHrB4?width=2400")),
            Model::API => (model.to_string(), String::from("https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9LWGU0ekxmSENqVlJTM2tmV0VzSy5wbmcifQ:supercell:SmOqSjpbIjqKqwrmZ2RWpEbwvBi1ERlMIp4Oe9fGI0g?width=2400")),
            Model::GUILD => (model.to_string(), Default::default()),
            Model::CHANNEL => (model.to_string(), Default::default()),
            Model::TOURNAMENT => (model.to_string(), String::from("https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9aY3Zxakt5TG91TDJVeU1BbkFCQi5wbmcifQ:supercell:FleTjqgzhQpseN715yWB6FF2EvJeI-8JtnalU_Db5Nc?width=2400")),
            Model::SYSTEM =>  (model.to_string(), Default::default()),
            Model::DEFAULT =>  (model.to_string(), Default::default()),
        };
        CreateEmbedAuthor::new(name).icon_url(icon_url)
    }

    fn thumbnail(&self, state: &State) -> String {
        match state{
            State::FAILURE => String::from("https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9mbkhRWjhzQmtkNUFkY2tzZTdTai5wbmcifQ:supercell:mCcCEDMJI8puCKKc2K9bBURE4tZem68vd5aMETOFjjw?width=2400"),
            State::SUCCESS => String::from("https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9iZUduOFpWaWpZYTduUXFKOEtDbi5wbmcifQ:supercell:QVmY9TjwRiZ77-CWw_lkKnpMrFbNbjHBZwalfHQ3KnE?width=2400"),
            State::INFO => String::from("https://cdn.discordapp.com/emojis/1187845402163167363.webp?size=4096&quality=lossless"),
            State::WARNING => String::from("https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9IWjFzZkUyNllLUW9hRWhlTlgyTi5wbmcifQ:supercell:CeCBNWeUn35mJJYWe4g5BMg9_gWf1l1D35idcw0RGXI?width=2400"),
        }
    }

    async fn log(
        &self,
        title: impl Into<String>,
        description: impl Into<String>,
        state: State,
        model: Model,
    ) -> Result<(), BotError> {
        let embed = CreateEmbed::default()
            .author(self.get_author_img(&model))
            .title(title)
            .description(format!(
                r#"**Action**:
{reason}
**Triggered by**
<@{id}>-`{id}`"#,
                reason = description.into(),
                id = self.author().id
            ))
            .timestamp(self.now())
            .thumbnail(self.thumbnail(&state))
            .colour(state as u32);
        let builder = CreateMessage::default().embed(embed);
        let channel = self.get_log_channel().await?;
        channel.send_message(self, builder).await?;
        Ok(())
    }
}

/// Creates an info log message in the current guild's designated log channel.
// pub async fn discord_log_info(
//     ctx: BotContext<'_>,
//     title: &str,
//     mut fields: Vec<(&str, &str, bool)>,
// ) -> Result<(), BotError> {
//     let guild_id = ctx
//         .guild_id()
//         .ok_or(anyhow!(
//             "Error sending info log: Attempted to perform an info log outside of a guild"
//         ))?
//         .to_string();

//     let log_channel = ChannelId::from_str(
//         &ctx.data()
//             .database
//             .get_config(&guild_id)
//             .await?
//             .ok_or(anyhow!(
//                 "Error sending info log: config not found for guild {}",
//                 guild_id
//             ))?
//             .log_channel_id,
//     )?;

//     info!("ℹ️ {}\n\n{:#?}", title, fields);

//     let now_string = format!(
//         "<t:{}:F>",
//         SystemTime::now()
//             .duration_since(SystemTime::UNIX_EPOCH)
//             .unwrap_or_default()
//             .as_secs()
//     );

//     fields.push(("Happened at", &now_string, false));

//     log_channel
//         .send_message(
//             ctx,
//             CreateMessage::default().content("").embed(
//                 CreateEmbed::new()
//                     .title(format!("ℹ️ {}", title))
//                     .fields(fields)
//                     .color(Color::BLURPLE),
//             ),
//         )
//         .await?;

//     Ok(())
// }

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
