pub mod model;

use crate::database::ConfigDatabase;
use crate::log::Log;
use crate::utils::discord::{modal, select_options};
use crate::{database::PgDatabase, utils::shorthand::BotContextExt, BotContext, BotError};
use anyhow::anyhow;
use async_recursion::async_recursion;
use futures::StreamExt;
use model::Mail;
use poise::serenity_prelude::{
    AutoArchiveDuration, ButtonStyle, ChannelType, CreateActionRow, CreateButton, CreateEmbed,
    CreateMessage, CreateThread, Mentionable, RoleId,
};
use poise::{serenity_prelude::UserId, Modal};
use poise::{CreateReply, ReplyHandle};

pub trait MailDatabase {
    type Error;
    async fn store(&self, mail: Mail) -> Result<(), Self::Error>;
    // async fn get_all(&self) -> Result<Option<Mail>, Self::Error>;
    async fn mark_read(&self, mail_id: i64) -> Result<(), Self::Error>;
    async fn unread(&self, user: UserId) -> Result<i64, Self::Error>;
    async fn get_all(&self, recipient: UserId) -> Result<Vec<Mail>, Self::Error>;
    async fn get_conversation(
        &self,
        sender: UserId,
        recipient: UserId,
    ) -> Result<Vec<Mail>, Self::Error>;
    async fn to_marshal(&self, mail: &mut Mail, role_id: RoleId) -> Result<(), Self::Error>;
}

impl MailDatabase for PgDatabase {
    type Error = BotError;
    async fn get_all(&self, recipient: UserId) -> Result<Vec<Mail>, Self::Error> {
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
            mail.mode as MailType
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

    async fn to_marshal(&self, mail: &mut Mail, role_id: RoleId) -> Result<(), Self::Error> {
        mail.marshal_type();
        mail.recipient = role_id.to_string();
        self.store(mail.clone()).await?;
        Ok(())
    }
}

pub trait MailBotCtx<'a> {
    async fn compose(
        &self,
        msg: &ReplyHandle<'_>,
        recipient: UserId,
        auto_subject: impl Into<Option<String>>,
    ) -> Result<(), BotError>;
    async fn inbox(&self, msg: &ReplyHandle<'_>) -> Result<(), BotError>;
    async fn mail_notification(&self) -> Result<(), BotError>;
    async fn unread(&self, user: UserId) -> Result<bool, BotError>;
    async fn get_conversation(&self, recipient: UserId) -> Result<Vec<Mail>, BotError>;
}

impl<'a> MailBotCtx<'a> for BotContext<'a> {
    async fn compose(
        &self,
        msg: &ReplyHandle<'_>,
        recipient: UserId,
        auto_subject: impl Into<Option<String>>,
    ) -> Result<(), BotError> {
        let embed = CreateEmbed::new()
            .title("Compose an mail")
            .description("Please press at the button below to compose a mail");
        let mail = match auto_subject.into() {
            None => {
                let modal = modal::<ComposeMail>(self, msg, embed).await?;
                Mail::new(
                    self.author().id.to_string(),
                    recipient.to_string(),
                    modal.subject,
                    modal.body,
                    None,
                )
                .await
            }
            Some(subject) => {
                let modal = modal::<ComposeMailWithoutSubject>(self, msg, embed).await?;
                Mail::new(
                    self.author().id.to_string(),
                    recipient.to_string(),
                    subject,
                    modal.body,
                    None,
                )
                .await
            }
        };
        self.data().database.store(mail).await?;
        let embed = CreateEmbed::new().title("Mail sent!").description(format!(
            "Your mail has been sent to {} successfully!\n You can safely dismiss this message!",
            recipient.mention()
        ));
        self.prompt(msg, embed, None).await?;
        Ok(())
    }

    async fn unread(&self, user: UserId) -> Result<bool, BotError> {
        let count = self.data().database.unread(user).await?;
        Ok(count > 0)
    }

    async fn mail_notification(&self) -> Result<(), BotError> {
        self.defer_ephemeral().await?;
        match self.data().database.unread(self.author().id).await? {
            0 => Ok(()),
            count @ _ => {
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
        const CHUNK: usize = 10;
        let mails = self.data().database.get_all(self.author().id).await?;
        if mails.is_empty() {
            let embed = CreateEmbed::new()
                .title("No unread mail")
                .description("You have no unread mail!");
            self.prompt(msg, embed, None).await?;
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
#[name = "Compose an mail"]
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
        .thumbnail(mail.sender(ctx).await?.avatar_url().unwrap_or_default());
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
                interactions.defer(ctx.http()).await?;
                break;
            }
            "reply" => {
                interactions.defer(ctx.http()).await?;
                ctx.compose(msg, mail.sender_id()?, mail.subject.clone())
                    .await?;
            }
            "report" => {
                interactions.defer(ctx.http()).await?;
                report(ctx, msg, mail).await?;
            }
            _ => {}
        }
    }
    Ok(())
}

async fn detail(ctx: &BotContext<'_>, mails: &[Mail]) -> Result<CreateEmbed, BotError> {
    let mut inbox = Vec::with_capacity(mails.len());
    for mail in mails {
        let sender = mail.sender(ctx).await?.mention();
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
        .title(format!("{}'s inbox", ctx.author().mention()))
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
    chunked_mail: &Vec<&[Mail]>,
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
        let selected = select_options(
            ctx,
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
            id @ _ => {
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

async fn report(ctx: &BotContext<'_>, msg: &ReplyHandle<'_>, mail: &Mail) -> Result<(), BotError> {
    const AUTO_ARCHIVE_DURATION: AutoArchiveDuration = AutoArchiveDuration::OneDay;
    let guild = ctx.guild().ok_or(anyhow!("Guild not found"))?.id;
    let marshal = ctx
        .data()
        .database
        .get_marshal_role(&guild.to_string())
        .await?;
    let log = ctx.get_log_channel().await?;
    let thread = CreateThread::new(mail.recipient_id()?.to_string())
        .kind(ChannelType::PublicThread)
        .name(mail.recipient(ctx).await?.name)
        .auto_archive_duration(AUTO_ARCHIVE_DURATION);
    let reply = {
        let sender = mail.sender(ctx).await?.mention();
        let recipient = mail.recipient(ctx).await?.mention();
        let embed = CreateEmbed::default()
            .title("A potential suspicious mail has been reported!")
            .description(format!(
                r#"From: {sender} `{sender_id}`
To: {recipient} `{recipient_id}`
Subject: {subject}
```
{body}
```
Sent at <t:{timestamp}:F>
Reported by: {recipient}.

"#,
                sender = sender,
                sender_id = mail.sender.clone(),
                recipient = recipient,
                recipient_id = mail.recipient.clone(),
                subject = mail.subject.clone(),
                body = mail.body.clone(),
                timestamp = mail.id,
            ))
            .timestamp(ctx.now());

        CreateMessage::new().embed(embed).content(format!(
            "{}",
            marshal.map_or_else(|| "".to_string(), |r| r.mention().to_string())
        ))
    };
    let thread_id = log.create_thread(ctx.http(), thread).await?;
    thread_id.send_message(ctx.http(), reply).await?;
    Ok(())
}
