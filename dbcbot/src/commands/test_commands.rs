use crate::database::Database;
use crate::log::{self, Log};
use crate::utils::shorthand::BotContextExt;
use crate::{api, BotContext, BotData, BotError};
use crate::api::{ApiResult, GameApi};
use super::CommandsContainer;
use anyhow::anyhow;

/// CommandsContainer for the User commands
pub struct TestCommands;

impl CommandsContainer for TestCommands {
    type Data = BotData;
    type Error = BotError;

    fn get_all() -> Vec<poise::Command<Self::Data, Self::Error>> {
        vec![battle_log(), match_image(), result_image(), profile_image()]
    }
}

use poise::serenity_prelude::{self, CreateAttachment};
use poise::{
  serenity_prelude::{CreateEmbed, CreateEmbedFooter},
  CreateReply,
};
use tracing::info;
/// Test command to get the battle log of a player
#[poise::command(slash_command)]
async fn battle_log(
    ctx: BotContext<'_>, 
    #[description="Tag of the player"] tag: String) -> Result<(), BotError> {
  ctx.defer().await?;
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

/// Test command to generate an image of a match between two players
#[poise::command(slash_command)]
async fn match_image(
    ctx: BotContext<'_>, 
    #[description="First user (in the left side)"] user1: serenity_prelude::User, 
    #[description="Second user (in the right side)"] user2: serenity_prelude::User
    ) -> Result<(), BotError> {
    ctx.defer().await?;
    let p1 = ctx.get_user_by_discord_id(user1.id.to_string()).await?.ok_or(anyhow!("User 1 not found."))?;
    let p2 = ctx.get_user_by_discord_id(user2.id.to_string()).await?.ok_or(anyhow!("User 2 not found."))?;
    let image_api = api::ImagesAPI::new()?;
    let image = match image_api.match_image(&p1, &p2).await{
        Ok(image) => image,
        Err(e) => {
            ctx.send(CreateReply::default().content("Error generating image. Please try again later.")).await?;
            ctx.log("Error generating image", format!("{e}") , log::State::FAILURE, log::Model::API).await?;
            return Ok(())
        }
    };
    let reply = {
        let embed = CreateEmbed::new()
            .title("Match image")
            .author(ctx.get_author_img(&log::Model::PLAYER))
            .description("Testing generating images of a match")
            .color(serenity_prelude::Colour::DARK_GOLD)
            .fields(vec![
                ("Player 1", format!("{}\n{}\n{}\n{}", p1.discord_name, p1.discord_id, p1.player_name, p1.player_tag), true),
                ("Player 2", format!("{}\n{}\n{}\n{}", p2.discord_name, p2.discord_id, p2.player_name, p2.player_tag), true),
            ]);
        CreateReply::default()
            .reply(true)
            .embed(embed)
            .attachment(CreateAttachment::bytes(image, "Test_match_image.png"))
    };
    ctx.send(reply).await?;
  Ok(())
}

/// Test command to generate an image of a match result between two players
#[poise::command(slash_command)]
async fn result_image(ctx: BotContext<'_>, 
    #[description="Winner of a match (in the left side)"] winner: serenity_prelude::User, 
    #[description="Eliminated player of a match (in the right side)"] loser: serenity_prelude::User
) -> Result<(), BotError> {
    ctx.defer().await?;
    let p1 = ctx.get_user_by_discord_id(winner.id.to_string()).await?.ok_or(anyhow!("Winner not found."))?;
    let p2 = ctx.get_user_by_discord_id(loser.id.to_string()).await?.ok_or(anyhow!("Loser not found."))?;
    let image_api = api::ImagesAPI::new()?;
    let image = match image_api.result_image(&p1, &p2).await{
        Ok(image) => image,
        Err(e) => {
            ctx.send(CreateReply::default().content("Error generating image. Please try again later.")).await?;
            ctx.log("Error generating image", format!("{e}") , log::State::FAILURE, log::Model::API).await?;
            return Ok(())
        }
    };
    let reply = {
        let embed = CreateEmbed::new()
            .title("Match image")
            .author(ctx.get_author_img(&log::Model::PLAYER))
            .description("Testing generating images of a match")
            .color(serenity_prelude::Colour::DARK_GOLD)
            .fields(vec![
                ("Player 1", format!("{}\n{}\n{}\n{}", p1.discord_name, p1.discord_id, p1.player_name, p1.player_tag), true),
                ("Player 2", format!("{}\n{}\n{}\n{}", p2.discord_name, p2.discord_id, p2.player_name, p2.player_tag), true),
            ]);
        CreateReply::default()
            .reply(true)
            .embed(embed)
            .attachment(CreateAttachment::bytes(image, "Test_match_image.png"))
    };
    ctx.send(reply).await?;
  Ok(())
}

/// Test command to get a player's profile
#[poise::command(slash_command)]
async fn profile_image(ctx: BotContext<'_>, 
    #[description="User to view profile"] user: serenity_prelude::User) -> Result<(), BotError> {
    ctx.defer().await?;
    let discord_id = user.id.to_string();
    let user = ctx.get_user_by_discord_id(discord_id.clone()).await?.ok_or(anyhow!("User not found."))?;
    let tournament_id = ctx
        .data()
        .database
        .get_active_tournaments_from_player(&ctx.author().id.to_string())
        .await?
        .get(0)
        .map_or_else(||"Not yet in a tournament".to_string(), |t| t.tournament_id.to_string());
    let image_api = api::ImagesAPI::new()?;
    let image = match image_api.profile_image(&user, tournament_id).await{
        Ok(image) => image,
        Err(e) => {
            ctx.send(CreateReply::default().content("Error generating image. Please try again later.")).await?;
            ctx.log("Error generating image", format!("{e}") , log::State::FAILURE, log::Model::API).await?;
            return Ok(())
        }
    };
    let reply = {
        let embed = CreateEmbed::new()
            .title("Match image")
            .author(ctx.get_author_img(&log::Model::PLAYER))
            .description("Testing generating images of a match")
            .color(serenity_prelude::Colour::DARK_GOLD)
            .fields(vec![
                ("Player 1", format!("{}\n{}\n{}\n{}", user.discord_name, user.discord_id, user.player_name, user.player_tag), true),
            ]);
        CreateReply::default()
            .reply(true)
            .embed(embed)
            .attachment(CreateAttachment::bytes(image, "Test_match_image.png"))
    };
    ctx.send(reply).await?;
    Ok(())
}