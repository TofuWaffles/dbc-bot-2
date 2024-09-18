use poise::serenity_prelude::{User, UserId};
use serde::{Deserialize, Serialize};

use crate::{BotContext, BotError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Mail {
    id: i64, //timestamp
    sender: String,
    recipient: String,
    subject: String,
    match_id: Option<String>,
    body: String,
    read: bool,
}
impl Mail {
    async fn new(
        sender: String,
        recipient: String,
        subject: String,
        body: String,
        match_id: impl Into<Option<String>>,
    ) -> Self {
        Self {
            id: chrono::Utc::now().timestamp(),
            sender,
            recipient,
            subject,
            body,
            match_id: match_id.into(),
            read: false,
        }
    }
    async fn recepient(&self, ctx: &BotContext<'_>) -> Result<User, BotError> {
        Ok(UserId::new(self.recipient.parse::<u64>()?)
            .to_user(ctx.http())
            .await?)
    }

    async fn sender(&self, ctx: &BotContext<'_>) -> Result<User, BotError> {
        Ok(UserId::new(self.sender.parse::<u64>()?)
            .to_user(ctx.http())
            .await?)
    }
}
