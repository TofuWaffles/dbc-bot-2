use std::str::FromStr;

use crate::{database::models::Selectable, utils::discord::DiscordTrait, BotContext, BotError};
use poise::serenity_prelude::{User, UserId};
use serde::{Deserialize, Serialize};
#[derive(Debug, PartialEq, Eq, sqlx::Type, Serialize, Deserialize, Clone)]
#[sqlx(type_name = "mail_type", rename_all = "snake_case")]
pub enum MailType {
    User,
    Marshal,
}

impl Default for MailType {
    fn default() -> Self {
        Self::User
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mail {
    pub id: i64, //timestamp
    pub sender: String,
    pub recipient: String,
    pub subject: String,
    #[serde(default)]
    pub match_id: String,
    pub body: String,
    pub read: bool,
    pub mode: MailType,
}

impl DiscordTrait for Mail {}

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
            match_id: match_id.into().unwrap_or_default(),
            read: false,
            mode: MailType::default(),
        }
    }

    #[inline]
    pub async fn recipient(&self, ctx: &BotContext<'_>) -> Result<User, BotError> {
        Ok(self.recipient_id()?.to_user(ctx.http()).await?)
    }

    #[inline]
    pub fn recipient_id(&self) -> Result<UserId, BotError> {
        Ok(UserId::from_str(self.recipient.as_str())?)
    }

    #[inline]
    pub async fn sender(&self, ctx: &BotContext<'_>) -> Result<User, BotError> {
        Ok(self.sender_id()?.to_user(ctx.http()).await?)
    }

    #[inline]
    pub fn sender_id(&self) -> Result<UserId, BotError> {
        Ok(UserId::from_str(self.sender.as_str())?)
    }

    #[inline]
    pub fn marshal_type(&mut self) {
        self.mode = MailType::Marshal;
    }
}

impl Selectable for Mail {
    #[inline]
    fn identifier(&self) -> String {
        self.id.to_string()
    }

    #[inline]
    fn label(&self) -> String {
        format!("{} - {}", self.subject.clone(), self.sender.clone())
    }
}
