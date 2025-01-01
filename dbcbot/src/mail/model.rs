use crate::{
    database::models::Selectable,
    utils::{discord::DiscordTrait, error::CommonError},
    BotContext, BotError,
};
use poise::serenity_prelude::{Mention, Mentionable, RoleId};
use poise::serenity_prelude::{Role, User, UserId};
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
#[derive(Debug, PartialEq, Eq, sqlx::Type, Serialize, Deserialize, Copy, Clone)]
#[sqlx(type_name = "mail_type", rename_all = "snake_case")]
pub enum MailType {
    User,
    Marshal,
}
#[derive(Debug, Clone, Copy)]
pub enum ActorId {
    User(UserId),
    Role(RoleId),
}

pub enum Actor {
    User(User),
    Marshal(Role),
}

impl From<UserId> for ActorId {
    fn from(user_id: UserId) -> Self {
        ActorId::User(user_id)
    }
}

impl From<RoleId> for ActorId {
    fn from(role_id: RoleId) -> Self {
        ActorId::Role(role_id)
    }
}

impl fmt::Display for ActorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActorId::User(user) => write!(f, "{}", user.to_string()),
            ActorId::Role(role) => write!(f, "{}", role.to_string()),
        }
    }
}

impl From<User> for Actor {
    fn from(user: User) -> Self {
        Actor::User(user)
    }
}

impl From<Role> for Actor {
    fn from(role: Role) -> Self {
        Actor::Marshal(role)
    }
}

impl ActorId {
    pub fn is_marshal(&self) -> bool {
        matches!(self, ActorId::Role(_))
    }
}

impl Default for MailType {
    fn default() -> Self {
        Self::User
    }
}

impl Actor {
    pub fn mention(&self) -> Mention {
        match self {
            Actor::User(user) => user.mention(),
            Actor::Marshal(role) => role.mention(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Actor::User(user) => user.name.clone(),
            Actor::Marshal(role) => role.name.clone(),
        }
    }

    pub fn id(&self) -> ActorId {
        match self {
            Actor::User(user) => ActorId::User(user.id),
            Actor::Marshal(role) => ActorId::Role(role.id),
        }
    }

    pub fn avatar_url(&self) -> String {
        match self {
            Actor::User(user) => user.avatar_url().unwrap_or_default(),
            Actor::Marshal(_) => String::default(),
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
        to_marshal: bool,
    ) -> Self {
        Self {
            id: chrono::Utc::now().timestamp(),
            sender,
            recipient,
            subject,
            body,
            match_id: match_id.into().unwrap_or_default(),
            read: false,
            mode: [MailType::User, MailType::Marshal][to_marshal as usize],
        }
    }

    #[inline]
    fn actor_id(&self, actor: &str) -> Result<ActorId, BotError> {
        match self.mode {
            MailType::User => Ok(UserId::from_str(actor)?.into()),
            MailType::Marshal => Ok(RoleId::from_str(actor)?.into()),
        }
    }

    #[inline]
    async fn actor(&self, ctx: &BotContext<'_>, actor: &str) -> Result<Actor, BotError> {
        Ok(match self.actor_id(actor)? {
            ActorId::User(user_id) => Actor::User(user_id.to_user(ctx.http()).await?),
            ActorId::Role(role_id) => {
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
    pub fn recipient_id(&self) -> Result<ActorId, BotError> {
        self.actor_id(self.recipient.as_str())
    }

    #[inline]
    pub async fn recipient(&self, ctx: &BotContext<'_>) -> Result<Actor, BotError> {
        self.actor(ctx, self.recipient.as_str()).await
    }

    #[inline]
    pub async fn sender(&self, ctx: &BotContext<'_>) -> Result<Actor, BotError> {
        Ok(self.actor(ctx, self.sender.as_str()).await?)
    }

    #[inline]
    pub fn sender_id(&self) -> Result<ActorId, BotError> {
        self.actor_id(self.sender.as_str())
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
