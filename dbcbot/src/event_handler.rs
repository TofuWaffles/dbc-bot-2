use crate::{
    database::ConfigDatabase,
    mail::{model::Mail, MailDatabase},
    utils::error::CommonError,
    BotData, BotError,
};
use anyhow::anyhow;
use poise::serenity_prelude::{
    self as serenity, ComponentInteraction, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateQuickModal, EditChannel, Mentionable
};
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
            match component.data.custom_id.as_str(){
                marshal_mail if marshal_mail.starts_with("marshal_mail") => {
                    let recipient_id = component
                        .data
                        .custom_id
                        .split("_")
                        .nth(2)
                        .map(|s| s.parse::<i64>())
                        .ok_or_else(|| anyhow!("Invalid marshal_mail id"))??;
                    handle_mail(ctx, data, component, recipient_id).await?;
                },
                "resolved" => {
                    handle_close_thread(ctx, component).await?;
                },
                _ => {}
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
    let Some(response) = mci.quick_modal(ctx, builder).await? else {
        let embed = CreateEmbed::default()
            .title("Error")
            .description("Invalid response")
            .color(0xff0000);
        let reply = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);
        let builder = CreateInteractionResponse::Message(reply);
        mci.create_response(ctx, builder).await?;
        return Ok(());
    };
    let [ref subject, ref body, ..] = response.inputs[..] else {
        panic!("This ain't happen")
    };
    let new_mail = Mail::new(
        marshal_role.to_string(),
        current_mail.sender,
        subject.clone(),
        body.clone(),
        mci.channel_id.to_string(),
        true,
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
    let builder = CreateInteractionResponseMessage::new().embed(embed);
    response
        .interaction
        .create_response(ctx, CreateInteractionResponse::Message(builder))
        .await?;
    Ok(())
}


async fn handle_close_thread(
    ctx: &serenity::Context,
    mci: &ComponentInteraction
) -> Result<(), BotError> {
    let channel = mci.channel_id;
    let name = channel.name(ctx).await?;
    if name.starts_with("[RESOLVED]") {
        let followup = CreateInteractionResponseFollowup::new()
            .content("This thread is already resolved")
            .ephemeral(true);
        mci.create_followup(ctx, followup).await?;
        return Ok(());
    } 
    let edited_channel = EditChannel::new().name(format!("[RESOLVED]{}", name));
    channel.edit(ctx, edited_channel).await?;
    mci.defer(ctx).await?;
    Ok(())
}