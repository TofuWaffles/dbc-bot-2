use poise::{
    serenity_prelude::{CreateEmbed, CreateEmbedFooter},
    CreateReply,
};
use tracing::info;

use crate::{
    api::{ApiResult, GameApi},
    BotContext, BotError,
};
#[poise::command(slash_command)]
pub async fn battle_log(ctx: BotContext<'_>, tag: String) -> Result<(), BotError> {
    ctx.defer_ephemeral().await?;
    let data = ctx.data().game_api.get_battle_log(&tag).await?;
    let logs = match data {
        ApiResult::Ok(battle_log) => battle_log,
        ApiResult::NotFound => {
            ctx.say("Player not found.").await?;
            return Ok(());
        }
        ApiResult::Maintenance => {
            ctx.say("API is currently under maintenance. Please try again later.")
                .await?;
            return Ok(());
        }
    };
    let log = &logs.items[0];
    info!("{:?}", log);
    let fields = vec![
        ("Mode", log.battle.mode.to_string(), true),
        ("Result", log.battle.result.to_string(), true),
    ];

    let embed = CreateEmbed::new()
        .description(format!("Battle log for player {}:", tag))
        .fields(fields)
        .footer(CreateEmbedFooter::new(log.battle_time.to_string()));
    let reply = {
        let mut reply = CreateReply::default();
        reply.embeds.push(embed);
        reply
    };
    ctx.send(reply).await?;
    Ok(())
}
