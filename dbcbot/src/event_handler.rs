use crate::{database::ConfigDatabase, mail::{model::Mail, MailDatabase}, utils::error::CommonError, BotContext, BotData, BotError};
use anyhow::anyhow;
use poise::serenity_prelude::{self as serenity, ButtonStyle, ComponentInteraction, CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, CreateQuickModal, Interaction};
use crate::utils::shorthand::ComponentInteractionExt;
pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, BotData, BotError>,
    data: &BotData,
) -> Result<(), BotError> {
    match event {
        serenity::FullEvent::InteractionCreate { interaction } => {
            let msg = interaction.as_message_component();
            if let Some(mci) = msg {
                match mci.data.custom_id.as_str() {
                    marshal_mail if marshal_mail.starts_with("marshal_mail") => {
                        let recipient_id = marshal_mail
                            .split("_")
                            .nth(2)
                            .map(|s| s.parse::<i64>())
                            .ok_or_else(|| anyhow!("Invalid marshal_mail id"))??;

                        handle_mail(ctx, data, mci, recipient_id).await?;
                    }
                    _ => {}
                }
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
    let marshal_role = data.database.get_marshal_role(&guild_id).await?.ok_or(CommonError::RoleNotExists("Marshal".to_string()))?;
    let builder = CreateQuickModal::new("Reply").short_field("Subject").paragraph_field("Body");
    if let Some(response) = mci.quick_modal( ctx,builder).await?.map(|r| r.inputs){
        let [ref subject, ref body, ..] = response[..] else {
            return Err(anyhow!("Invalid response"));
        };
        let new_mail = Mail::new(marshal_role.to_string(), current_mail.sender, subject.clone(), body.clone(), None);
        
    let embed = {
        CreateEmbed::default()
            .title(new_mail.subject.clone())
            .description(new_mail.body.clone())
            .fields(vec![
                ("Sender", new_mail.sender.clone(), true),
                ("At", format!("<t:{}:F>", new_mail.id), true),
            ])
    };
    let btn_id = format!("marshal_mail_{}", new_mail.id);
    let buttons = vec![CreateActionRow::Buttons(vec![CreateButton::new(btn_id)
        .label("Respond")
        .style(ButtonStyle::Danger)])];
    data.database.store(new_mail).await?;
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .embed(embed)
            .components(buttons)
    );
    mci.create_response(ctx, response).await?;
    }   
    Ok(())
}
