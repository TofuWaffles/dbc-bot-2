use poise::serenity_prelude::{User, UserId};
use serde::{Deserialize, Serialize};

use crate::{database::models::Selectable, BotContext, BotError};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mail {
    pub id: i64, //timestamp
    pub sender: String,
    pub recipient: String,
    pub subject: String,
    pub match_id: Option<String>,
    pub body: String,
    pub read: bool,
}
impl Mail {
    pub async fn new(
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
    pub async fn recepient(&self, ctx: &BotContext<'_>) -> Result<User, BotError> {
        Ok(UserId::new(self.recipient.parse::<u64>()?)
            .to_user(ctx.http())
            .await?)
    }

    pub async fn sender(&self, ctx: &BotContext<'_>) -> Result<User, BotError> {
        Ok(UserId::new(self.sender.parse::<u64>()?)
            .to_user(ctx.http())
            .await?)
    }
}

impl Selectable for Mail {
    fn identifier(&self) -> String {
        self.id.to_string()
    }

    fn label(&self) -> String {
        self.subject.clone()
    }
}