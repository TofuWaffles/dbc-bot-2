use crate::{
    database::models::Selectable,
    utils::{discord::DiscordTrait, error::CommonError},
    BotContext, BotError,
};
use poise::serenity_prelude::{Mention, Mentionable, RoleId};
use poise::serenity_prelude::{Role, User, UserId};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
#[derive(Debug, PartialEq, Eq, sqlx::Type, Serialize, Deserialize, Clone)]
#[sqlx(type_name = "mail_type", rename_all = "snake_case")]
pub enum MailType {
    User,
    Marshal,
}
#[derive(Debug, Clone, Copy)]
pub enum RecipientId {
    User(UserId),
    Role(RoleId),
}

pub enum Recipient {
    User(User),
    Marshal(Role),
}

impl From<UserId> for RecipientId {
    fn from(user_id: UserId) -> Self {
        RecipientId::User(user_id)
    }
}

impl From<RoleId> for RecipientId {
    fn from(role_id: RoleId) -> Self {
        RecipientId::Role(role_id)
    }
}

impl fmt::Display for RecipientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecipientId::User(user) => write!(f, "{}", user.to_string()),
            RecipientId::Role(role) => write!(f, "{}", role.to_string()),
        }
    }
}

impl From<User> for Recipient {
    fn from(user: User) -> Self {
        Recipient::User(user)
    }
}

impl From<Role> for Recipient {
    fn from(role: Role) -> Self {
        Recipient::Marshal(role)
    }
}

impl RecipientId {
    pub fn is_marshal(&self) -> bool {
        matches!(self, RecipientId::Role(_))
    }
}

impl Default for MailType {
    fn default() -> Self {
        Self::User
    }
}

impl Recipient {
    pub fn mention(&self) -> Mention {
        match self {
            Recipient::User(user) => user.mention(),
            Recipient::Marshal(role) => role.mention(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Recipient::User(user) => user.name.clone(),
            Recipient::Marshal(role) => role.name.clone(),
        }
    }

    pub fn id(&self) -> RecipientId {
        match self {
            Recipient::User(user) => RecipientId::User(user.id),
            Recipient::Marshal(role) => RecipientId::Role(role.id),
        }
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
    pub fn new(
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
    pub async fn recipient(&self, ctx: &BotContext<'_>) -> Result<Recipient, BotError> {
        Ok(match self.recipient_id()? {
            RecipientId::User(user_id) => Recipient::User(user_id.to_user(ctx.http()).await?),
            RecipientId::Role(role_id) => {
                let guild = ctx.guild().ok_or(CommonError::NotInAGuild)?;
                guild
                    .roles
                    .get(&role_id)
                    .ok_or(CommonError::RoleNotExists(role_id.to_string()))?
                    .clone()
                    .into()
            }
        })
    }

    #[inline]
    pub fn recipient_id(&self) -> Result<RecipientId, BotError> {
        match self.mode {
            MailType::User => Ok(UserId::from_str(self.recipient.as_str())?.into()),
            MailType::Marshal => Ok(RoleId::from_str(self.recipient.as_str())?.into()),
        }
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
