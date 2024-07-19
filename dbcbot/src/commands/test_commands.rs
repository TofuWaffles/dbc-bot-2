<<<<<<< HEAD
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
        vec![battle_log(), match_image()]
    }
}

use poise::serenity_prelude::{self, CreateAttachment};
use poise::{
  serenity_prelude::{CreateEmbed, CreateEmbedFooter},
  CreateReply,
};
use tracing::info;

#[poise::command(slash_command)]
async fn battle_log(ctx: BotContext<'_>, tag: String) -> Result<(), BotError> {
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

#[poise::command(slash_command)]
async fn match_image(ctx: BotContext<'_>, user1: serenity_prelude::User, user2: serenity_prelude::User) -> Result<(), BotError> {
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
=======
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
        vec![battle_log(), match_image()]
    }
}

use poise::serenity_prelude::{self, CreateAttachment};
use poise::{
  serenity_prelude::{CreateEmbed, CreateEmbedFooter},
  CreateReply,
};
use tracing::info;

#[poise::command(slash_command)]
async fn battle_log(ctx: BotContext<'_>, tag: String) -> Result<(), BotError> {
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

#[poise::command(slash_command)]
async fn match_image(ctx: BotContext<'_>, user1: serenity_prelude::User, user2: serenity_prelude::User) -> Result<(), BotError> {
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
>>>>>>> bdb70236c68496a534164a78cf20d5436eba400c
}