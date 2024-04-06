use poise::CreateReply;

use crate::{
    api::BrawlStarsApi, commands::checks::is_manager, database::{Database, PgDatabase}, tournament_model::SingleElimTournament, BotError, Context, Data
};

use super::CommandsContainer;

/// CommandsContainer for the Manager commands
pub struct UserCommands;

impl CommandsContainer<PgDatabase, SingleElimTournament, BrawlStarsApi> for UserCommands {
    fn get_commands_list(
    ) -> Vec<poise::Command<crate::Data<PgDatabase, SingleElimTournament, BrawlStarsApi>, BotError>> {
        vec![]
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
    match user {
        Some(_) => {
            ctx.send(
                CreateReply::default()
                    .content("You have already registered your profile.")
                    .ephemeral(true),
            )
            .await?;
        }
        None => {
            // Check with the BS api here
            ctx.data().database.create_user(&user_id, &player_tag).await?;
            ctx.send(
                CreateReply::default()
                    .content("You have successfully registered your profile.")
                    .ephemeral(true),
            )
            .await?;
        }
    };

    Ok(())
}
