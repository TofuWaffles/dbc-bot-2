use crate::{database::{models::GuildConfig, Database}, BotContext, BotError};
use anyhow::anyhow;
pub trait BotContextExt {
    async fn get_config(&self) -> Result<Option<GuildConfig>, BotError>;
}

impl BotContextExt for BotContext<'_>{
    async fn get_config(&self) -> Result<Option<GuildConfig>, BotError>{
      let guild_id = self.guild_id().ok_or(anyhow!("Not running this in a guild"))?;
      let config = self.data().database.get_config(&guild_id.to_string()).await?;
      Ok(config)
    }
}