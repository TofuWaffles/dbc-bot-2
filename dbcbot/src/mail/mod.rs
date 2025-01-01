pub mod model;
use std::str::FromStr;

use crate::database::ConfigDatabase;
use crate::log::Log;
use crate::utils::error::CommonError::{self, *};
use crate::utils::shorthand::{BotComponent, ComponentInteractionExt};
use crate::{database::PgDatabase, utils::shorthand::BotContextExt, BotContext, BotError};
use async_recursion::async_recursion;
use futures::StreamExt;
use model::{Mail, MailType, ActorId};
use poise::serenity_prelude::{
    AutoArchiveDuration, ButtonStyle, ChannelId, ChannelType, Colour, ComponentInteractionCollector, CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse, CreateMessage, CreateThread, EditMessage, Guild, GuildChannel, Mentionable
};
use poise::{serenity_prelude::UserId, Modal};
use poise::{CreateReply, ReplyHandle};
use tracing::info;
const AUTO_ARCHIVE_DURATION: AutoArchiveDuration = AutoArchiveDuration::OneDay;
pub trait MailDatabase {
    async fn get_mail_by_id(&self, mail_id: i64) -> Result<Mail, Self::Error>;
    type Error;
    async fn store(&self, mail: Mail) -> Result<(), Self::Error>;
    // async fn get_all(&self) -> Result<Option<Mail>, Self::Error>;
    async fn mark_read(&self, mail_id: i64) -> Result<(), Self::Error>;
    async fn unread(&self, user: UserId) -> Result<i64, Self::Error>;
    async fn get_all_mails(&self, recipient: UserId) -> Result<Vec<Mail>, Self::Error>;
    async fn get_conversation(
        &self,
        sender: UserId,
        recipient: UserId,
    ) -> Result<Vec<Mail>, Self::Error>;
}

impl MailDatabase for PgDatabase {
    type Error = BotError;
    async fn get_all_mails(&self, recipient: UserId) -> Result<Vec<Mail>, Self::Error> {
        let mails = sqlx::query_as!(
            Mail,
            r#"
            SELECT 
                id, sender, recipient, subject, match_id, body, read, mode as "mode: MailType"
            FROM mail 
            WHERE recipient = $1
            ORDER BY id DESC
            "#,
            recipient.to_string()
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(mails)
    }

    async fn get_mail_by_id(&self, mail_id: i64) -> Result<Mail, Self::Error> {
        let mail = sqlx::query_as!(
            Mail,
            r#"
            SELECT 
                id, sender, recipient, subject, match_id, body, read, mode as "mode: MailType"
            FROM mail
            WHERE id = $1
            "#,
            mail_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(mail)
    }

    async fn unread(&self, user: UserId) -> Result<i64, Self::Error> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) FROM mail
            WHERE $1 = recipient
                AND read = false
            "#,
            user.to_string()
        )
        .fetch_one(&self.pool)
        .await?
        .count
        .unwrap_or(0);
        Ok(count)
    }

    async fn mark_read(&self, mail_id: i64) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            UPDATE mail
            SET read = true
            WHERE id = $1
            "#,
            mail_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn store(&self, mail: Mail) -> Result<(), Self::Error> {
        sqlx::query!(
            r#"
            INSERT INTO mail (id, sender, recipient, subject, match_id, body, read, mode)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            mail.id,
            mail.sender,
            mail.recipient,
            mail.subject,
            mail.match_id,
            mail.body,
            mail.read,
            mail.mode as MailType,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_conversation(
        &self,
        sender: UserId,
        recipient: UserId,
    ) -> Result<Vec<Mail>, Self::Error> {
        let mails = sqlx::query_as!(
            Mail,
            r#"
            SELECT 
                id, sender, recipient, subject, match_id, body, read, mode as "mode: MailType"
            FROM mail
            WHERE (sender = $1  AND recipient = $2)
                OR (recipient = $1 AND sender = $2 )
        "#,
            sender.to_string(),
            recipient.to_string()
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(mails)
    }
}

pub trait MailBotCtx<'a> {
    type Error;
    async fn compose(
        &self,
        msg: &ReplyHandle<'_>,
        embed: CreateEmbed,
        recipient_id: impl Into<ActorId>,
        auto_subject: impl Into<Option<String>>,
        to_marshals: bool
    ) -> Result<i64, BotError>;
    async fn inbox(&self, msg: &ReplyHandle<'_>) -> Result<(), BotError>;
    async fn mail_notification(&self) -> Result<(), BotError>;
    async fn unread(&self, user: UserId) -> Result<bool, BotError>;
    async fn get_conversation(&self, recipient: UserId) -> Result<Vec<Mail>, BotError>;
    async fn send_to_marshal(
        &self,
        msg: &ReplyHandle<'_>,
        embed: CreateEmbed,
    ) -> Result<(), Self::Error>;
}

impl<'a> MailBotCtx<'a> for BotContext<'a> {
    type Error = BotError;
    async fn compose(
        &self,
        msg: &ReplyHandle<'_>,
        embed: CreateEmbed,
        recipient_id: impl Into<ActorId>,
        auto_subject: impl Into<Option<String>>,
        to_marshals: bool
    ) -> Result<i64, BotError> {
        let recipient_id: ActorId = recipient_id.into();
        let mut mail = match auto_subject.into() {
            None => {
                let modal = self.components().modal::<ComposeMail>(msg, embed).await?;
                Mail::new(
                    self.author().id.to_string(),
                    recipient_id.to_string(),
                    modal.subject,
                    modal.body,
                    None,
                    to_marshals,
                )
            }
            Some(subject) => {
                let modal = self
                    .components()
                    .modal::<ComposeMailWithoutSubject>(msg, embed)
                    .await?;
                Mail::new(
                    self.author().id.to_string(),
                    recipient_id.to_string(),
                    subject,
                    modal.body,
                    None,
                    to_marshals,
                    
                )
            }
        };
        if recipient_id.is_marshal() {
            mail.marshal_type();
        }

        let recipient = mail.recipient(self).await?;
        let id = mail.id;
        self.data().database.store(mail).await?;

        let embed = CreateEmbed::new().title("Mail sent!").description(format!(
            "Your mail has been sent to {} successfully!\n You can safely dismiss this message!",
            recipient.mention()
        ));
        self.components().prompt(msg, embed, None).await?;
        Ok(id)
    }

    async fn unread(&self, user: UserId) -> Result<bool, BotError> {
        let count = self.data().database.unread(user).await?;
        Ok(count > 0)
    }

    async fn mail_notification(&self) -> Result<(), BotError> {
        self.defer_ephemeral().await?;
        match self.data().database.unread(self.author().id).await? {
            0 => Ok(()),
            count => {
                let embed = CreateEmbed::new().title("Unread mail").description(format!(
                    "You have {} unread mail(s)! Choose mail button in the Menu to access.",
                    count
                ));
                let reply = CreateReply::default().embed(embed).ephemeral(true);
                self.send(reply).await?;
                Ok(())
            }
        }
    }

    async fn inbox(&self, msg: &ReplyHandle<'_>) -> Result<(), BotError> {
        self.components().prompt(msg, CreateEmbed::new().description("Getting inbox"), None).await?;
        const CHUNK: usize = 10;
        let mails = self.data().database.get_all_mails(self.author().id).await?;
        if mails.is_empty() {
            let embed = CreateEmbed::new()
                .title("No unread mail")
                .description("You have no unread mail!");
            self.components().prompt(msg, embed, None).await?;
            return Ok(());
        }
        let chunked_mail: Vec<&[Mail]> = mails.chunks(CHUNK).collect();
        inbox_helper(self, msg, &chunked_mail).await?;
        Ok(())
    }

    async fn get_conversation(&self, recipient: UserId) -> Result<Vec<Mail>, BotError> {
        self.data()
            .database
            .get_conversation(self.author().id, recipient)
            .await
    }

    async fn send_to_marshal(
        &self,
        msg: &ReplyHandle<'_>,
        embed: CreateEmbed,
    ) -> Result<(), Self::Error> {
        let guild_id = self.guild_id().ok_or(NotInAGuild)?;
        let role = self
            .data()
            .database
            .get_marshal_role(&guild_id)
            .await?
            .ok_or(RoleNotExists("Marshal".to_string()))?;
        let id = self.compose(msg, embed, role, None, true).await?;
        let embed = CreateEmbed::new().title("Mail sent!").description( "Your mail has been sent to the marshal team successfully!\n You can safely dismiss this message!");
        self.components().prompt(msg, embed, None).await?;
        let mail = self.data().database.get_mail_by_id(id).await?;
        let thread_id = self.author().id.to_string();
        let embed = {
            CreateEmbed::default()
                .title(mail.subject.clone())
                .description(mail.body.clone())
                .fields(vec![
                    ("Sender", format!("<@{}>",mail.sender.to_string()), true),
                    ("At", format!("<t:{}:F>", mail.id), true),
                ])
        };
        let guild_id = self.guild_id().ok_or(CommonError::NotInAGuild)?;
        let channels = guild_id.channels(self.http()).await?;
        match channels.get(&ChannelId::from_str(&thread_id).unwrap()) {
            Some(thread) => {
                return open_thread(self, embed, role, thread.clone(), mail.id).await;
            }
            None => {
                let log_channel = self.get_log_channel().await?;
                let thread = CreateThread::new(thread_id)
                    .kind(ChannelType::PublicThread)
                    .auto_archive_duration(AUTO_ARCHIVE_DURATION);
                let thread = log_channel.create_thread(self.http(), thread).await?;
                return open_thread(self, embed, role, thread, mail.id).await;
            }
        };
    }
}
#[derive(Debug, Modal)]
#[name = "Compose a mail"]
struct ComposeMail {
    #[name = "Subject"]
    #[placeholder = "The subject of the mail"]
    subject: String,

    #[name = "Body"]
    #[paragraph]
    #[placeholder = "The body of the mail"]
    body: String,
}

#[derive(Debug, Modal)]
#[name = "Compose a mail"]
struct ComposeMailWithoutSubject {
    #[name = "Body"]
    #[paragraph]
    #[placeholder = "The body of the mail"]
    body: String,
}

#[async_recursion]
async fn mail_page(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    mail: &Mail,
) -> Result<(), BotError> {
    let embed = CreateEmbed::new()
        .title(&mail.subject)
        .description(format!(
            "{}\n{}\nSent at <t:{}:F>",
            mail.sender(ctx).await?.mention(),
            &mail.body,
            mail.id
        ))
        .thumbnail(mail.sender(ctx).await?.avatar_url());
    ctx.data().database.mark_read(mail.id).await?;
    let buttons = CreateActionRow::Buttons(vec![
        CreateButton::new("reply")
            .label("Reply")
            .style(ButtonStyle::Success),
        CreateButton::new("back")
            .label("Back")
            .style(ButtonStyle::Secondary),
        CreateButton::new("report")
            .label("Report to Marshals")
            .style(ButtonStyle::Danger),
    ]);
    let reply = CreateReply::default()
        .embed(embed)
        .components(vec![buttons]);
    msg.edit(*ctx, reply).await?;
    let mut ic = ctx.create_interaction_collector(msg).await?;
    while let Some(interactions) = &ic.next().await {
        match interactions.data.custom_id.as_str() {
            "back" => {
                interactions
                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                    .await?;
            }
            "reply" => {
                interactions.defer(ctx.http()).await?;
                let embed = CreateEmbed::default()
                    .title("Compose a reply mail to the sender")
                    .description("Press the button below to compose a reply mail to the sender!");
                match mail.mode{
                    MailType::Marshal => {
                        return ctx.send_to_marshal(msg, embed).await;
                    }
                    _ => {
                        ctx.compose(msg, embed, mail.sender_id()?, mail.subject.clone(), false).await?;
                    }
                }                
            }
            "report" => {
                interactions.defer(ctx.http()).await?;
                let mut mail = mail.clone();
                report(ctx, msg, &mut mail).await?;
            }
            _ => {}
        }
    }
    Ok(())
}

async fn detail(ctx: &BotContext<'_>, mails: &[Mail]) -> Result<CreateEmbed, BotError> {
    let mut inbox = Vec::with_capacity(mails.len());
    for mail in mails {
        let sender = match mail.sender(ctx).await{
            Ok(sender) => sender.mention().to_string(),
            Err(_) => format!("Unknown user with id {}", mail.sender),
        };
        inbox.push(format!(
            r#"{read_status} | From {sender} 
**{subject}**
-# Sent at <t:{time_sent}:F>"#,
            read_status = if mail.read { "✉️" } else { "📩" },
            time_sent = mail.id,
            subject = mail.subject,
        ))
    }
    Ok(CreateEmbed::default()
        .title(format!("{}'s inbox", ctx.author().name))
        .description(format!(
            "There are {} mail(s) in this page!\nSelect a mail to read it\n{}",
            mails.len(),
            inbox.join("\n")
        ))
        .timestamp(ctx.now()))
}

#[async_recursion]
async fn inbox_helper(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    chunked_mail: &[&[Mail]],
) -> Result<(), BotError> {
    let (prev, next) = (String::from("prev"), String::from("next"));
    let mut page_number: usize = 0;
    let total = chunked_mail.len();
    let buttons = CreateActionRow::Buttons(vec![
        CreateButton::new(prev.clone())
            .label("⬅️")
            .style(ButtonStyle::Primary),
        CreateButton::new(next.clone())
            .label("➡️")
            .style(ButtonStyle::Primary),
    ]);
    loop {
        let selected = ctx
            .components()
            .select_options(
                msg,
                detail(ctx, chunked_mail[page_number]).await?,
                vec![buttons.clone()],
                chunked_mail[page_number],
            )
            .await?;
        match selected.as_str() {
            "prev" => {
                page_number = page_number.saturating_sub(1);
            }
            "next" => {
                page_number = (page_number + 1).min(total - 1);
            }
            id => {
                let mail = chunked_mail[page_number]
                    .iter()
                    .find(|mail| mail.id.to_string() == id)
                    .unwrap()
                    .to_owned();
                mail_page(ctx, msg, &mail).await?;
            }
        }
    }
}

async fn report(ctx: &BotContext<'_>, msg: &ReplyHandle<'_>, mail: &mut Mail) -> Result<(), BotError> {
    let embed = {
        let sender = mail.sender(ctx).await?;
        let recipient = mail.recipient(ctx).await?.mention();
        {
            CreateEmbed::default()
                .title("A potential suspicious mail has been reported!")
                .description(format!(
                    r#"
Subject: {subject}
```
{body}
```
Sent at <t:{timestamp}:F>
Reported by: {recipient}.

"#,
                    subject = mail.subject.clone(),
                    body = mail.body.clone(),
                    timestamp = mail.id,
                ))
                .fields(vec![
                    ("From", format!("{}`{}`", sender.mention(), sender.id()), true),
                    (
                        "To",
                        format!("{}`{}`", recipient.mention(), recipient),
                        true,
                    ),
                ])
                .color(Colour::RED)
                .timestamp(ctx.now())
        }
    };
    let log = ctx.get_log_channel().await?;
    let reporter_name= ctx.author().name.clone();
    let thread = CreateThread::new(reporter_name)
        .kind(ChannelType::PublicThread)
        .auto_archive_duration(AUTO_ARCHIVE_DURATION);  
    let channel = log.create_thread(ctx.http(), thread).await?;
    open_thread(ctx, embed, mail.recipient_id()?, channel, mail.id ).await?;
    ctx.components().prompt(msg,
        CreateEmbed::new()
            .title("This mail has been reported!")
            .description("You can safely dismiss this mail. The marshals will keep you up-to-date about this report!"), 
            None)
        .await?;
    Ok(())
}

#[async_recursion]
async fn open_thread(
    ctx: &BotContext<'_>,
    embed: CreateEmbed,
    _id: impl Into<ActorId> + Send + 'async_recursion,
    thread: GuildChannel,
    mail_id: i64,
) -> Result<(), BotError> {
    let btn_id = format!("marshal_mail_{}", mail_id);
    let buttons = vec![CreateActionRow::Buttons(vec![CreateButton::new(btn_id)
        .label("Respond")
        .style(ButtonStyle::Danger)])];
    let reply = CreateMessage::new()
        .embed(embed)
        .components(buttons.clone());
    thread.send_message(ctx.http(), reply).await?;
    Ok(())
}
