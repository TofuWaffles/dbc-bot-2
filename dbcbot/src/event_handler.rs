use crate::{
    database::ConfigDatabase,
    mail::{model::Mail, MailDatabase},
    utils::{error::CommonError, shorthand::ComponentInteractionExt},
    BotData, BotError,
};
use anyhow::anyhow;
use poise::serenity_prelude::{
    self as serenity, ComponentInteraction,
    CreateEmbed, CreateInteractionResponseFollowup, CreateQuickModal, Mentionable,
};
use tracing::info;
pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, BotData, BotError>,
    data: &BotData,
) -> Result<(), BotError> {
    match event {
        serenity::FullEvent::InteractionCreate {
            interaction: serenity::Interaction::Component(component),
        } => {
            if component.data.custom_id.starts_with("marshal_mail") {
                let recipient_id = component
                    .data
                    .custom_id
                    .split("_")
                    .nth(2)
                    .map(|s| s.parse::<i64>())
                    .ok_or_else(|| anyhow!("Invalid marshal_mail id"))??;

                handle_mail(ctx, data, component, recipient_id).await?;
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_mail(
    ctx: &serenity::Context,
    data: &BotData,
    mci: &ComponentInteraction,
    mail_id: i64,
) -> Result<(), BotError> {
    let current_mail = data.database.get_mail_by_id(mail_id).await?;
    let guild_id = mci.guild_id.ok_or_else(|| anyhow!("Guild ID not found"))?;
    let marshal_role = data
        .database
        .get_marshal_role(&guild_id)
        .await?
        .ok_or(CommonError::RoleNotExists("Marshal".to_string()))?;
    let builder = CreateQuickModal::new("Reply")
        .short_field("Subject")
        .paragraph_field("Body");
    if let Some(response) = mci.quick_modal(ctx, builder).await?.map(|r| r.inputs) {
        let [ref subject, ref body, ..] = response[..] else {
            return Err(anyhow!("Invalid response"));
        };
        let new_mail = Mail::new(
            marshal_role.to_string(),
            current_mail.sender,
            subject.clone(),
            body.clone(),
            None,
            true
        );

        let embed = {
            CreateEmbed::default()
                .title(new_mail.subject.clone())
                .description(new_mail.body.clone())
                .fields(vec![
                    ("Responded by", mci.user.mention().to_string(), true),
                    ("Responder's id,", mci.user.id.to_string(), true),
                    ("At", format!("<t:{}:F>", new_mail.id), true),
                ])
        };
        data.database.store(new_mail).await?;
        let response = CreateInteractionResponseFollowup::new().embed(embed);
        info!("Interaction reaches here!");
        mci.create_followup(ctx, response).await?;
        mci.acknowledge(ctx).await?;
    }
    Ok(())
}
