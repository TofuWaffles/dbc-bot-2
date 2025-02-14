use super::CommandsContainer;
use crate::api::images::ImagesAPI;
use crate::api::APIResult;
use crate::database::models::{BrawlMap, Mode};
use crate::database::{Database, TournamentDatabase};
use crate::log::{self, Log};
use crate::mail::MailBotCtx;
use crate::utils::shorthand::{BotComponent, BotContextExt};
use crate::{BotContext, BotData, BotError};
use anyhow::anyhow;

/// CommandsContainer for the User commands
pub struct TestCommands;

impl CommandsContainer for TestCommands {
    type Data = BotData;
    type Error = BotError;

    fn get_all() -> Vec<poise::Command<Self::Data, Self::Error>> {
        vec![
            // battle_log(),
            // match_image(),
            // result_image(),
            // profile_image(),
            // choose_brawler_command(),
            // choose_map_command(),
            // choose_gamemode_command(),
            // send_mail(),
            // add_maps(),
            // csv(),
        ]
    }
}

use poise::serenity_prelude::{self, CreateAttachment};
use poise::{
    serenity_prelude::{CreateEmbed, CreateEmbedFooter},
    CreateReply,
};
/// Test command to get the battle log of a player
#[poise::command(slash_command)]
async fn battle_log(
    ctx: BotContext<'_>,
    #[description = "Tag of the player"] tag: String,
) -> Result<(), BotError> {
    ctx.defer().await?;
    let data = ctx.data().apis.brawl_stars.get_battle_log(&tag).await?;
    let logs = match data {
        APIResult::Ok(battle_log) => battle_log,
        APIResult::NotFound => {
            ctx.say("Player not found.").await?;
            return Ok(());
        }
        APIResult::Maintenance => {
            ctx.say("API is currently under maintenance. Please try again later.")
                .await?;
            return Ok(());
        }
    };
    let log = &logs.items[0];
    println!("{:?}", log);
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
    #[description = "First user (in the left side)"] user1: serenity_prelude::User,
    #[description = "Second user (in the right side)"] user2: serenity_prelude::User,
    #[description = "Publish the result"] ephemeral: bool,
) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default()
                .ephemeral(ephemeral)
                .embed(CreateEmbed::new().title("Running the test")),
        )
        .await?;
    let p1 = ctx
        .get_player_from_discord_id(user1.id.to_string())
        .await?
        .ok_or(anyhow!("User 1 not found."))?;
    let p2 = ctx
        .get_player_from_discord_id(user2.id.to_string())
        .await?
        .ok_or(anyhow!("User 2 not found."))?;
    let image_api = ImagesAPI::new();
    let image = match image_api.match_image(&p1, &p2).await {
        Ok(image) => image,
        Err(e) => {
            ctx.components()
                .prompt(
                    &msg,
                    CreateEmbed::new()
                        .title("An error has occured!")
                        .description(e.to_string()),
                    None,
                )
                .await?;
            return Ok(());
        }
    };
    let reply = {
        let embed = CreateEmbed::new()
            .title("Match image")
            .author(ctx.get_author_img(&log::Model::PLAYER))
            .description("Testing generating images of a match")
            .color(serenity_prelude::Colour::DARK_GOLD)
            .fields(vec![
                (
                    "Player 1",
                    format!(
                        "{}\n{}\n{}\n{}",
                        p1.discord_name, p1.discord_id, p1.player_name, p1.player_tag
                    ),
                    true,
                ),
                (
                    "Player 2",
                    format!(
                        "{}\n{}\n{}\n{}",
                        p2.discord_name, p2.discord_id, p2.player_name, p2.player_tag
                    ),
                    true,
                ),
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
async fn result_image(
    ctx: BotContext<'_>,
    #[description = "Winner of a match (in the left side)"] winner: serenity_prelude::User,
    #[description = "Eliminated player of a match (in the right side)"]
    loser: serenity_prelude::User,
    #[description = "Score"] score: String,
    #[description = "Publish the result"] ephemeral: bool,
) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default()
                .ephemeral(ephemeral)
                .embed(CreateEmbed::new().title("Running the test")),
        )
        .await?;
    let p1 = ctx
        .get_player_from_discord_id(winner.id.to_string())
        .await?
        .ok_or(anyhow!("Winner not found."))?;
    let p2 = ctx
        .get_player_from_discord_id(loser.id.to_string())
        .await?
        .ok_or(anyhow!("Loser not found."))?;
    let image_api = ImagesAPI::new();
    let image = match image_api.result_image(&p1, &p2, &score).await {
        Ok(image) => image,
        Err(e) => {
            ctx.components()
                .prompt(
                    &msg,
                    CreateEmbed::new()
                        .title("An error has occured!")
                        .description(e.to_string()),
                    None,
                )
                .await?;
            return Ok(());
        }
    };
    let reply = {
        let embed = CreateEmbed::new()
            .title("Match image")
            .author(ctx.get_author_img(&log::Model::PLAYER))
            .description("Testing generating images of a match")
            .color(serenity_prelude::Colour::DARK_GOLD)
            .fields(vec![
                (
                    "Player 1",
                    format!(
                        "{}\n{}\n{}\n{}",
                        p1.discord_name, p1.discord_id, p1.player_name, p1.player_tag
                    ),
                    true,
                ),
                (
                    "Player 2",
                    format!(
                        "{}\n{}\n{}\n{}",
                        p2.discord_name, p2.discord_id, p2.player_name, p2.player_tag
                    ),
                    true,
                ),
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
async fn profile_image(
    ctx: BotContext<'_>,
    #[description = "User to view profile"] user: serenity_prelude::User,
    #[description = "Publish the result"] ephemeral: bool,
) -> Result<(), BotError> {
    ctx.defer().await?;
    let msg = ctx
        .send(
            CreateReply::default()
                .ephemeral(ephemeral)
                .embed(CreateEmbed::new().title("Running the test")),
        )
        .await?;
    let discord_id = user.id.to_string();
    let user = ctx
        .get_player_from_discord_id(discord_id.clone())
        .await?
        .ok_or(anyhow!("User not found."))?;
    let tournament_id = ctx
        .data()
        .database
        .get_active_tournaments_from_player(&ctx.author().id)
        .await?
        .first()
        .map_or_else(|| "None".to_string(), |t| t.tournament_id.to_string());
    let image_api = ImagesAPI::new();
    let image = match image_api.profile_image(&user, tournament_id).await {
        Ok(image) => image,
        Err(e) => {
            ctx.components()
                .prompt(
                    &msg,
                    CreateEmbed::new()
                        .title("An error has occured!")
                        .description(e.to_string()),
                    None,
                )
                .await?;
            return Ok(());
        }
    };
    let reply = {
        let embed = CreateEmbed::new()
            .title("Match image")
            .author(ctx.get_author_img(&log::Model::PLAYER))
            .description("Testing generating images of a match")
            .color(serenity_prelude::Colour::DARK_GOLD)
            .fields(vec![(
                "Player 1",
                format!(
                    "{}\n{}\n{}\n{}",
                    user.discord_name, user.discord_id, user.player_name, user.player_tag
                ),
                true,
            )]);
        CreateReply::default()
            .reply(true)
            .embed(embed)
            .attachment(CreateAttachment::bytes(image, "Test_match_image.png"))
    };
    ctx.send(reply).await?;
    Ok(())
}

/// Test command to choose a brawler
#[poise::command(slash_command)]
async fn choose_brawler_command(ctx: BotContext<'_>) -> Result<(), BotError> {
    ctx.defer().await?;
    let msg = ctx.reply("Choose a brawler").await?;
    let brawler = ctx.components().brawler_selection(&msg).await?;
    ctx.say(format!("You chose {}", brawler.name)).await?;
    Ok(())
}

/// Test command to choose a brawler
#[poise::command(slash_command)]
async fn choose_map_command(ctx: BotContext<'_>, mode: Mode) -> Result<(), BotError> {
    ctx.defer().await?;
    let msg = ctx.reply("Test choosing map").await?;
    let map = ctx.components().map_selection(&msg, &mode).await?;
    let reply = {
        let embed = CreateEmbed::default()
            .title(map.name.to_string())
            .description(format!(
                "Environment: **{}**\nMode: **{}**\nAvailability: **{}**",
                map.environment.name,
                map.game_mode.name,
                ["Yes", "No"][(!map.disabled) as usize]
            ))
            .image(map.image_url)
            .thumbnail(map.game_mode.image_url)
            .footer(CreateEmbedFooter::new("Provided by Brawlify"));
        CreateReply::default().embed(embed).components(vec![])
    };
    msg.edit(ctx, reply).await?;
    Ok(())
}

/// Test command to choose a game mode
#[poise::command(slash_command)]
async fn choose_gamemode_command(ctx: BotContext<'_>) -> Result<(), BotError> {
    ctx.defer().await?;
    let msg = ctx.reply("Choose a game mode").await?;
    let mode = ctx.components().mode_selection(&msg).await?;
    let reply = {
        let embed = CreateEmbed::default()
            .title(mode.name.to_string())
            .description(format!(
                "Description: **{}**\nAvailability: **{}**",
                mode.description,
                ["Yes", "No"][(!mode.disabled) as usize]
            ))
            .thumbnail(mode.image_url)
            .footer(CreateEmbedFooter::new("Provided by Brawlify"));
        CreateReply::default().embed(embed).components(vec![])
    };
    msg.edit(ctx, reply).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn send_mail(
    ctx: BotContext<'_>,
    recipient: serenity_prelude::User,
) -> Result<(), BotError> {
    let msg = ctx.reply("Test sending a mail").await?;
    let embed = CreateEmbed::default()
        .title("Mail")
        .description("This is a test mail.")
        .footer(CreateEmbedFooter::new("This is a test mail."));
    ctx.compose(&msg, embed, recipient.id, None, false).await?;
    Ok(())
}

/// Test add all maps to the database
#[poise::command(slash_command)]
pub async fn add_maps(ctx: BotContext<'_>) -> Result<(), BotError> {
    let reply = {
        let embed = CreateEmbed::default()
            .title("Adding maps to the database")
            .description("This command will add all maps to the database.")
            .footer(CreateEmbedFooter::new("This may take a while."));
        CreateReply::default()
            .ephemeral(true)
            .embed(embed)
            .reply(true)
    };
    let msg = ctx.send(reply).await?;
    let raw = ctx.data().apis.brawlify.get_maps().await?;
    let mut maps = match raw.handler(&ctx, &msg).await? {
        Some(maps) => maps,
        None => {
            return ctx
                .components()
                .prompt(
                    &msg,
                    CreateEmbed::default().description("No maps were added!"),
                    None,
                )
                .await
        }
    };

    while let Some(map) = maps.pop() {
        let brawl_map = BrawlMap::from(map);
        ctx.data().database.add_map(&brawl_map).await?;
    }
    ctx.components()
        .prompt(
            &msg,
            CreateEmbed::default().description("All maps were added!"),
            None,
        )
        .await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn csv(ctx: BotContext<'_>) -> Result<(), BotError> {
    #[derive(Debug, poise::Modal)]
    #[name = "CSV Test"]
    struct CSV {
        #[name = "CSV text here"]
        #[paragraph]
        text: String,
    }
    let msg = ctx
        .send(CreateReply::default().content("Prepare CSV"))
        .await?;
    let embed = CreateEmbed::default()
        .title("CSV")
        .description("Here is the CSV file");
    let res = ctx.components().modal::<CSV>(&msg, embed).await?;
    let attachment: CreateAttachment =
        CreateAttachment::bytes(res.text.replace(" ", "\n").as_bytes(), "test.csv");
    ctx.send(CreateReply::default().attachment(attachment))
        .await?;
    Ok(())
}
