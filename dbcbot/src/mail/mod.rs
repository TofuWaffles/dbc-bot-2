pub mod model;
use crate::database::{ConfigDatabase, TournamentDatabase};
use crate::log::Log;
use crate::utils::error::CommonError::{self, *};
use crate::utils::shorthand::BotComponent;
use crate::{database::PgDatabase, utils::shorthand::BotContextExt, BotContext, BotError};
use anyhow::anyhow;
use async_recursion::async_recursion;
use futures::StreamExt;
use model::{Actor, ActorId, Mail, MailType};
use poise::serenity_prelude::{
    Attachment, AutoArchiveDuration, ButtonStyle, ChannelId, ChannelType, Colour, CreateActionRow,
    CreateButton, CreateEmbed, CreateInteractionResponse, CreateMessage, CreateThread, EditChannel,
    GuildChannel, Mentionable,
};
use poise::{serenity_prelude::UserId, Modal};
use poise::{CreateReply, ReplyHandle};
use std::str::FromStr;
use std::vec;
const AUTO_ARCHIVE_DURATION: AutoArchiveDuration = AutoArchiveDuration::OneDay;
pub trait MailDatabase {
    async fn get_all_sent_mails(&self, sender: UserId) -> Result<Vec<Mail>, Self::Error>;
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

    async fn get_all_sent_mails(&self, sender: UserId) -> Result<Vec<Mail>, Self::Error> {
        let mails = sqlx::query_as!(
            Mail,
            r#"
            SELECT 
                id, sender, recipient, subject, match_id, body, read, mode as "mode: MailType"
            FROM mail 
            WHERE sender = $1
            ORDER BY id DESC
            "#,
            sender.to_string()
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
            ORDER BY id ASC
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
    async fn outbox(&self, msg: &ReplyHandle<'_>) -> Result<(), BotError>;
    type Error;
    async fn compose(
        &self,
        msg: &ReplyHandle<'_>,
        embed: CreateEmbed,
        recipient_id: impl Into<ActorId>,
        auto_subject: impl Into<Option<String>>,
        to_marshals: bool,
    ) -> Result<i64, BotError>;
    async fn inbox(&self, msg: &ReplyHandle<'_>) -> Result<(), BotError>;
    async fn mail_notification(&self) -> Result<(), BotError>;
    async fn unread(&self, user: UserId) -> Result<bool, BotError>;
    async fn get_conversation(&self, recipient: UserId) -> Result<Vec<Mail>, BotError>;
    async fn send_to_marshal(
        &self,
        msg: &ReplyHandle<'_>,
        embed: CreateEmbed,
        attachments: Vec<Attachment>,
        channel_id: Option<String>,
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
        to_marshals: bool,
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
        if let Actor::User(u) = recipient {
            let t = &self
                .data()
                .database
                .get_active_tournaments_from_player(&u.id)
                .await?[0];
            let noti_channel = t.notification_channel(&self).await?;
            let content = format!(
                "{}, you have a new mail! Choose mail button in the Menu to access.",
                u.mention().to_string()
            );
            let reply = CreateMessage::default().content(content);
            noti_channel.send_message(&self, reply).await?;
        }
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
        self.components()
            .prompt(
                msg,
                CreateEmbed::new().description("Getting your inbox"),
                None,
            )
            .await?;
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
        inbox_helper(self, msg, &chunked_mail, false).await?;
        Ok(())
    }

    async fn outbox(&self, msg: &ReplyHandle<'_>) -> Result<(), BotError> {
        self.components()
            .prompt(
                msg,
                CreateEmbed::new().description("Getting your outbox"),
                None,
            )
            .await?;
        const CHUNK: usize = 10;
        let mails = self
            .data()
            .database
            .get_all_sent_mails(self.author().id)
            .await?;
        if mails.is_empty() {
            let embed = CreateEmbed::new()
                .title("No mails in outbox!")
                .description("You have not sent any mails! This outbox is empty!");
            self.components().prompt(msg, embed, None).await?;
            return Ok(());
        }
        let chunked_mail: Vec<&[Mail]> = mails.chunks(CHUNK).collect();
        inbox_helper(self, msg, &chunked_mail, true).await?;
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
        attachments: Vec<Attachment>,
        channel_id: Option<String>,
    ) -> Result<(), Self::Error> {
        let guild_id = self.guild_id().ok_or(NotInAGuild)?;
        let role = self
            .data()
            .database
            .get_marshal_role(&guild_id)
            .await?
            .ok_or(RoleNotExists("Marshal".to_string()))?;
        let id = self.compose(msg, embed, role, None, true).await?;
        let mail = self.data().database.get_mail_by_id(id).await?;
        let thread_name = self.author().name.clone();
        let embed = {
            CreateEmbed::default()
                .title(mail.subject.clone())
                .description(mail.body.clone())
                .fields(vec![
                    ("Sender", format!("<@{}>", mail.sender.to_string()), true),
                    ("At", format!("<t:{}:F>", mail.id), true),
                ])
        };
        match channel_id {
            Some(id) => {
                let guild_id = self.guild_id().ok_or(CommonError::NotInAGuild)?;
                let guild = self.http().get_guild(guild_id).await?;
                let thread_data = guild.get_active_threads(self).await?;
                match thread_data
                    .threads
                    .iter()
                    .find(|thread| thread.id == ChannelId::from_str(&id).unwrap())
                {
                    Some(thread) => {
                        return open_thread(
                            self,
                            embed,
                            role,
                            attachments,
                            thread.clone(),
                            mail.id,
                        )
                        .await;
                    }
                    None => {
                        let thread = create_thread(self, thread_name).await?;
                        return open_thread(self, embed, role, attachments, thread, mail.id).await;
                    }
                };
            }
            None => {
                let thread = create_thread(self, thread_name).await?;
                return open_thread(self, embed, role, attachments, thread, mail.id).await;
            }
        };

        async fn create_thread(
            ctx: &BotContext<'_>,
            thread_name: String,
        ) -> Result<GuildChannel, BotError> {
            let guild_id = ctx.guild_id().ok_or(CommonError::NotInAGuild)?;
            let config = ctx
                .data()
                .database
                .get_config(&guild_id)
                .await?
                .ok_or(anyhow!("No mail channel set yet"))?;
            let mail_channel = config.mail_channel(ctx).await?;
            let thread = CreateThread::new(thread_name)
                .kind(ChannelType::PublicThread)
                .auto_archive_duration(AUTO_ARCHIVE_DURATION);
            Ok(mail_channel.create_thread(ctx.http(), thread).await?)
        }
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
    outbox: bool,
) -> Result<(), BotError> {
    let embed = CreateEmbed::new()
        .title(&mail.subject)
        .description(&mail.body)
        .color(Colour::DARK_GOLD)
        .fields(vec![
            if outbox {
                ("To", mail.recipient(ctx).await?.mention().to_string(), true)
            } else {
                ("From", mail.sender(ctx).await?.mention().to_string(), true)
            },
            ("Sent at", format!("<t:{}:F>", mail.id), true),
        ])
        .thumbnail(mail.sender(ctx).await?.avatar_url());
    if !outbox {
        ctx.data().database.mark_read(mail.id).await?;
    }
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
    let reply = CreateReply::default().embed(embed).components(if !outbox {
        vec![buttons]
    } else {
        vec![]
    });
    msg.edit(*ctx, reply).await?;
    let mut ic = ctx.create_interaction_collector(msg).await?;
    while let Some(interactions) = &ic.next().await {
        match interactions.data.custom_id.as_str() {
            "back" => {
                interactions
                    .create_response(ctx, CreateInteractionResponse::Acknowledge)
                    .await?;
                ctx.inbox(msg).await?;
            }
            "reply" => {
                interactions.defer(ctx.http()).await?;
                let embed = CreateEmbed::default()
                    .title("Compose a reply mail to the sender")
                    .description("Press the button below to compose a reply mail to the sender!");
                let channel_id = if mail.match_id.is_empty() {
                    None
                } else {
                    Some(mail.match_id.clone())
                };
                match mail.mode {
                    MailType::Marshal => {
                        return ctx.send_to_marshal(msg, embed, vec![], channel_id).await;
                    }
                    _ => {
                        ctx.compose(msg, embed, mail.sender_id()?, mail.subject.clone(), false)
                            .await?;
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

async fn detail(
    ctx: &BotContext<'_>,
    mails: &[Mail],
    outbox: bool,
) -> Result<CreateEmbed, BotError> {
    let mut inbox = Vec::with_capacity(mails.len());
    if outbox {
        for mail in mails {
            let recipient = match mail.recipient(ctx).await {
                Ok(recipient) => recipient.mention().to_string(),
                Err(_) => format!("Unknown user with id {}", mail.recipient),
            };
            inbox.push(format!(
                r#"{read_status} | To {recipient}
    **{subject}**
    -# Sent at <t:{time_sent}:F>"#,
                read_status = if mail.read { "✉️" } else { "📩" },
                time_sent = mail.id,
                subject = mail.subject,
                recipient = recipient,
            ))
        }
    } else {
        for mail in mails {
            let sender = match mail.sender(ctx).await {
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
    }
    Ok(CreateEmbed::default()
        .title(format!(
            "{}'s {}",
            ctx.author().name,
            ["Inbox", "Outbox"][outbox as usize]
        ))
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
    outbox: bool,
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
        // CreateButton::new("outbox")
        //     .label("Outbox")
        //     .emoji(ReactionType::from('📤'))
        //     .style(ButtonStyle::Secondary),
    ]);
    loop {
        let selected = ctx
            .components()
            .select_options(
                msg,
                detail(ctx, chunked_mail[page_number], outbox).await?,
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
            "outbox" => {
                ctx.outbox(msg).await?;
            }
            id => {
                let mail = chunked_mail[page_number]
                    .iter()
                    .find(|mail| mail.id.to_string() == id)
                    .unwrap()
                    .to_owned();
                mail_page(ctx, msg, &mail, outbox).await?;
            }
        }
    }
}

async fn report(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
    mail: &mut Mail,
) -> Result<(), BotError> {
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
                    (
                        "From",
                        format!("{}`{}`", sender.mention(), sender.id()),
                        true,
                    ),
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
    let reporter_name = ctx.author().name.clone();
    let thread = CreateThread::new(reporter_name)
        .kind(ChannelType::PublicThread)
        .auto_archive_duration(AUTO_ARCHIVE_DURATION);
    let channel = log.create_thread(ctx.http(), thread).await?;
    open_thread(ctx, embed, mail.recipient_id()?, vec![], channel, mail.id).await?;
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
    attachments: Vec<Attachment>,
    thread: GuildChannel,
    mail_id: i64,
) -> Result<(), BotError> {
    resolve_check(ctx, &mut thread.clone()).await?;
    let btn_id = format!("marshal_mail_{}", mail_id);
    let buttons = vec![CreateActionRow::Buttons(vec![
        CreateButton::new(btn_id)
            .label("Respond")
            .style(ButtonStyle::Danger),
        CreateButton::new("resolved")
            .label("Resolved")
            .style(ButtonStyle::Success),
    ])];
    let reply = match attachments.len() {
        0 => CreateMessage::new()
            .content("There is no attachment in this mail!")
            .embed(embed)
            .components(buttons.clone()),
        1 => {
            let attachment = attachments[0].clone();
            CreateMessage::new()
                .content("There is an attachment in this mail!")
                .embed(embed.image(attachment.url.clone()))
                .components(buttons.clone())
        }
        2.. => {
            // Split the attachments to separate the first one from the rest
            if let Some((first, remaining)) = attachments.split_first() {
                let mut embeds: Vec<CreateEmbed> = remaining
                    .iter()
                    .map(|att| {
                        CreateEmbed::default()
                            .url("https://discord.gg/brawlstars")
                            .image(att.url.clone())
                    })
                    .collect();
                let new_embed = embed
                    .image(first.url.clone())
                    .url("https://discord.gg/brawlstars");
                embeds.insert(0, new_embed);
                CreateMessage::new()
                    .content(format!(
                        "There are {} attachments in this mail!",
                        attachments.len()
                    ))
                    .components(buttons.clone())
                    .embeds(embeds)
            } else {
                CreateMessage::new()
                    .embed(embed)
                    .components(buttons.clone())
            }
        }
    };
    thread.send_message(ctx.http(), reply).await?;
    Ok(())
}

async fn resolve_check(ctx: &BotContext<'_>, thread: &mut GuildChannel) -> Result<(), BotError> {
    if thread.name().starts_with("[RESOLVED]") {
        let edited_thread = EditChannel::new().name(thread.name().replace("[RESOLVED]", ""));
        thread.edit(ctx.http(), edited_thread).await?;
    }
    Ok(())
}
