use crate::{database::models::Selectable, BotContext, BotError};
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
    pub async fn recipient(&self, ctx: &BotContext<'_>) -> Result<User, BotError> {
        Ok(UserId::new(self.recipient.parse::<u64>()?)
            .to_user(ctx.http())
            .await?)
    }

    pub fn recipient_id(&self) -> Result<UserId, BotError> {
        Ok(UserId::new(self.recipient.parse::<u64>()?))
    }

    pub async fn sender(&self, ctx: &BotContext<'_>) -> Result<User, BotError> {
        Ok(UserId::new(self.sender.parse::<u64>()?)
            .to_user(ctx.http())
            .await?)
    }

    pub fn sender_id(&self) -> Result<UserId, BotError> {
        Ok(UserId::new(self.sender.parse::<u64>()?))
    }

    pub fn marshal_type(&mut self) {
        self.mode = MailType::Marshal;
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
