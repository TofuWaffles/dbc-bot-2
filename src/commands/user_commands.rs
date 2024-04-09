use poise::CreateReply;

use crate::{
    api::{ApiResult, GameApi},
    database::Database,
    BotData, BotError, Context,
};

use super::CommandsContainer;

/// CommandsContainer for the Manager commands
pub struct UserCommands;

impl CommandsContainer for UserCommands {
    type Data = BotData;
    type Error = BotError;

    fn get_commands_list() -> Vec<poise::Command<Self::Data, Self::Error>> {
        vec![menu(), register()]
    }
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn menu(ctx: Context<'_>) -> Result<(), BotError> {
    let user_id = ctx.author().id.to_string();

    let user = ctx.data().database.get_user(&user_id).await?;
    match user {
        Some(_) => {
            todo!()
        }
        None => {
            ctx.send(
                CreateReply::default()
                    .content("You have not registered your profile yet. Please register first with the /register command.")
                    .ephemeral(true),
            )
            .await?;
        }
    };

    Ok(())
}

#[poise::command(slash_command, prefix_command, guild_only)]
async fn register(ctx: Context<'_>, player_tag: String) -> Result<(), BotError> {
    let user_id = ctx.author().id.to_string();

    let user = ctx.data().database.get_user(&user_id).await?;
    if user.is_some() {
        ctx.send(
            CreateReply::default()
                .content("You have already registered your profile.")
                .ephemeral(true),
        )
        .await?;

        return Ok(());
    }

    let api_result = ctx.data().game_api.get_player(&player_tag).await?;
    match api_result {
        ApiResult::Ok(player) => {
            ctx.send(
                CreateReply::default()
                    .content(format!(
                        "You have successfully registered your profile. {:?}",
                        player
                    ))
                    .ephemeral(true),
            )
            .await?;
            // ctx.data().database.create_user(&user_id, &player_tag).await?;
        }
        ApiResult::NotFound => {
            ctx.send(
                CreateReply::default()
                    .content("A profile with that tag was not found. Please ensure that you have entered the correct tag.")
                    .ephemeral(true),
            )
            .await?;
        }
        ApiResult::Maintenance => {
            ctx.send(
                CreateReply::default()
                    .content("The Brawl Stars API is currently undergoing maintenance. Please try again later.")
                    .ephemeral(true),
            )
            .await?;
        }
    }

    Ok(())
}
