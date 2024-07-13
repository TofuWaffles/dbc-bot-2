use crate::utils::discord::{select_channel, select_role, splash};
use crate::utils::shorthand::BotContextExt;
use crate::{
    commands::checks::{is_config_set, is_manager},
    database::{
        models::{Match, Player, TournamentStatus},
        Database,
    },
    log::{self, discord_log_info},
    BotContext, BotData, BotError,
};
use anyhow::anyhow;

use poise::Modal;
use poise::{
    serenity_prelude::{
        self as serenity, Colour, CreateActionRow, CreateButton, CreateEmbed
    },
    CreateReply, ReplyHandle,
};
use tracing::{error, info, instrument};

use super::CommandsContainer;

/// CommandsContainer for the Manager commands.
pub struct ManagerCommands;

impl CommandsContainer for ManagerCommands {
    type Data = BotData;
    type Error = BotError;

    fn get_all() -> Vec<poise::Command<Self::Data, Self::Error>> {
        vec![
            set_config_slash(),
            create_tournament_slash(),
            start_tournament_slash(),
            manager_menu(),
        ]
    }
}
/// Set the configuration for a guild
///
/// This command is used to set the configuration for a guild. The configuration includes the marshal role, announcement channel, notification channel, and log channel.
///
/// - Marshal Role: The role that is able to manage the tournament system. Akin to tournament
/// marshals.
/// - Announcement Channel: The channel where the bot will announce the start and end of tournaments.
/// and other important announcements.
/// - Notification Channel: The channel where the bot will send notifications to players about their
/// progress and matches.
/// - Log Channel: The channel where the bot will log all the actions it takes.
#[poise::command(slash_command, prefix_command, guild_only, check = "is_manager")]
#[instrument]
async fn set_config_slash(
    ctx: BotContext<'_>,
    #[description = "This role can access tournament monitor commands!"]
    marshal_role: serenity::Role,
    #[description = "This channel announces winner for each match!"]
    announcement_channel: serenity::Channel,
    #[description = "This channel announces winner for each match!"]
    notification_channel: serenity::Channel,
    #[description = "This channel logs activities"] log_channel: serenity::Channel,
) -> Result<(), BotError> {
    set_config(
        ctx,
        marshal_role,
        announcement_channel,
        notification_channel,
        log_channel,
    )
    .await
}

/// Create a new tournament.
///
#[poise::command(slash_command, prefix_command, guild_only, check = "is_manager")]
#[instrument]
async fn create_tournament_slash(
    ctx: BotContext<'_>,
    #[description = "Tournament name"] name: String,
    #[description = "Role for the tournament"] role: serenity::Role,
) -> Result<(), BotError> {
    create_tournament(ctx, name, role).await
}

/// Start a tournament.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    check = "is_manager",
    check = "is_config_set"
)]
#[instrument]
async fn start_tournament_slash(
    ctx: BotContext<'_>,
    tournament_id: i32,
    map: Option<String>,
    win_required: Option<i32>,
) -> Result<(), BotError> {
    let map = map.unwrap_or(String::default());
    start_tournament(ctx, tournament_id, map, win_required).await
}

async fn set_config(
    ctx: BotContext<'_>,
    marshal_role: serenity::Role,
    announcement_channel: serenity::Channel,
    notification_channel: serenity::Channel,
    log_channel: serenity::Channel,
) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default()
                .content("Setting the configuration...")
                .ephemeral(true),
        )
        .await?;
    let announcement_channel_id = match announcement_channel.guild() {
        Some(guild) => guild.id.to_string(),
        None => {
            ctx.send(
                CreateReply::default()
                    .content("Please enter a valid server channel as the announcement channel")
                    .ephemeral(true),
            )
            .await?;
            error!("Invalid announcement channel entered by {}", ctx.author());
            return Ok(());
        }
    };

    let notification_channel_id = match notification_channel.guild() {
        Some(guild) => guild.id.to_string(),
        None => {
            ctx.send(
                CreateReply::default()
                    .content("Please enter a valid server channel as the notification channel")
                    .ephemeral(true),
            )
            .await?;
            error!("Invalid notification channel entered by {}", ctx.author());
            return Ok(());
        }
    };

    let log_channel_id = match log_channel.guild() {
        Some(guild) => guild.id.to_string(),
        None => {
            ctx.send(
                CreateReply::default()
                    .content("Please enter a valid server channel as the log channel")
                    .ephemeral(true),
            )
            .await?;
            error!("Invalid log channel entered by {}", ctx.author());
            return Ok(());
        }
    };

    let marshal_role_id = marshal_role.id.to_string();

    ctx.data()
        .database
        .set_config(
            &ctx.guild_id()
                .ok_or(anyhow!("This command must be used within a server"))?
                .to_string(),
            &marshal_role_id,
            &announcement_channel_id,
            &notification_channel_id,
            &log_channel_id,
        )
        .await?;

    msg.edit(ctx,
        CreateReply::default()
            .content("Successfully set the configuration. You can run the same command again to update the configuration.")
            .ephemeral(true),
    )
    .await?;

    info!(
        "Set the configuration for guild {}",
        ctx.guild_id().unwrap().to_string()
    );
    log::discord_log_info(ctx, "MANAGER CONFIGURATION SET SUCCESSFULLY!", vec![]).await?;

    Ok(())
}

/// Create a new tournament.
async fn create_tournament(ctx: BotContext<'_>, name: String, role: serenity::Role) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().unwrap().to_string();
    let role_id = role.id.to_string();
    let new_tournament_id = ctx
        .data()
        .database
        .create_tournament(&guild_id, &name, None, role_id)
        .await?;

    ctx.send(
        CreateReply::default()
            .content(format!(
                "Successfully created tournament with id {}",
                new_tournament_id
            ))
            .ephemeral(true),
    )
    .await?;

    discord_log_info(
        ctx,
        "A new tournament has been created.",
        vec![
            ("Tournament ID", &new_tournament_id.to_string(), false),
            ("Tournament name", &name, false),
            ("Created by", &ctx.author().name, false),
        ],
    )
    .await?;

    info!(
        "Created tournament {} for guild {}",
        new_tournament_id, guild_id
    );

    Ok(())
}

async fn start_tournament(
    ctx: BotContext<'_>,
    tournament_id: i32,
    map: String,
    wins_required: Option<i32>,
) -> Result<(), BotError> {
    let wins_required = match wins_required {
        Some(wins) => {
            if wins < 1 {
                ctx.send(CreateReply::default().content("Aborting operation: the number of required wins must not be less than 1!").ephemeral(true)).await?;
                return Ok(());
            } else {
                wins
            }
        }
        None => 2,
    };

    let guild_id = ctx.guild_id().unwrap().to_string();

    let tournament = match ctx
        .data()
        .database
        .get_tournament(&guild_id, tournament_id)
        .await?
    {
        Some(tournament) => tournament,
        None => {
            ctx.send(
                CreateReply::default()
                    .content("Tournament not found")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    match tournament.status {
        TournamentStatus::Pending => (),
        _ => {
            ctx.send(
                CreateReply::default()
                    .content(format!(
                        "Tournament with ID {} either has already started or has already ended.",
                        tournament_id
                    ))
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    }

    let tournament_players = ctx
        .data()
        .database
        .get_tournament_players(tournament_id)
        .await?;

    if tournament_players.len() < 2 {
        ctx.send(
            CreateReply::default()
                .content(format!(
                    "There are not enough players to start the tournament with ID {}.",
                    tournament_id
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let rounds_count = (tournament_players.len() as f64).log2().ceil() as i32;

    let matches = generate_matches_new_tournament(tournament_players, tournament_id).await?;

    let matches_count = matches.len();

    for bracket in matches {
        ctx.data()
            .database
            .create_match(
                bracket.tournament_id,
                bracket.round,
                bracket.sequence_in_round,
                bracket.player_1_type,
                bracket.player_2_type,
                bracket.discord_id_1.as_deref(),
                bracket.discord_id_2.as_deref(),
            )
            .await?
    }

    ctx.data()
        .database
        .set_tournament_status(tournament_id, TournamentStatus::Started)
        .await?;

    ctx.data()
        .database
        .set_rounds(tournament_id, rounds_count)
        .await?;

    ctx.data().database.set_map(tournament_id, &map).await?;

    ctx.send(CreateReply::default()
             .content(format!("Successfully started tournament with ID {}.\n\nTotal number of matches in the first round (including byes): {}", tournament_id, matches_count))
             .ephemeral(true)
        )
        .await?;

    discord_log_info(
        ctx,
        "A tournament has started",
        vec![
            ("Tournament ID", &tournament_id.to_string(), false),
            ("Tournament name", &tournament.name, false),
            ("Rounds", &rounds_count.to_string(), false),
            ("Number of matches", &matches_count.to_string(), false),
            ("Wins required per match", &wins_required.to_string(), false),
            ("Started by", &ctx.author().name, false),
        ],
    )
    .await?;

    Ok(())
}

/// Contains the logic for generating matches for a newly started tournament.
async fn generate_matches_new_tournament(
    tournament_players: Vec<Player>,
    tournament_id: i32,
) -> Result<Vec<Match>, BotError> {
    let rounds_count = (tournament_players.len() as f64).log2().ceil() as u32;

    let matches_count = 2_u32.pow(rounds_count - 1);

    let mut matches = Vec::new();

    for i in 0..matches_count {
        // Guaranteed to have a player
        let player_1 = &tournament_players[i as usize];
        // Not guaranteed to have a player, this would be a bye round if there is no player
        let player_2 = &tournament_players.get(matches_count as usize + i as usize);

        matches.push(Match::new(
            Match::generate_id(tournament_id, 1, (i + 1) as i32),
            tournament_id,
            1,
            (i + 1) as i32,
            Some(player_1.discord_id.to_owned()),
            player_2
                .as_ref()
                .map(|player_2| player_2.discord_id.to_owned()),
        ));
    }

    Ok(matches)
}
/// Marshal menu command.
#[poise::command(slash_command, prefix_command, guild_only, check = "is_manager")]
async fn manager_menu(ctx: BotContext<'_>) -> Result<(), BotError> {
    ctx.defer_ephemeral().await?;
    let embed = CreateEmbed::default()
        .title("Manager Menu")
        .description(
            r#"Select an option from the menu below.
🛠️: Set configurations for the tournament.
➕: Create a new tournament.        
▶️: Start a tournament.
"#,
        )
        .color(Colour::GOLD);
    let components = vec![CreateActionRow::Buttons(vec![
        CreateButton::new("conf")
            .style(serenity::ButtonStyle::Primary)
            .label("🛠️"),
        CreateButton::new("create")
            .style(serenity::ButtonStyle::Primary)
            .label("➕"),
        CreateButton::new("start")
            .style(serenity::ButtonStyle::Primary)
            .label("▶️"),
    ])];
    let builder = CreateReply::default()
        .embed(embed)
        .components(components)
        .reply(true);
    let msg = ctx.send(builder).await?;
    while let Some(mci) = serenity::ComponentInteractionCollector::new(ctx)
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(120))
        .await
    {
        match mci.data.custom_id.as_str() {
            "conf" => {
                mci.defer(ctx.http()).await?;
                return step_by_step_config(&ctx, &msg).await;
            }
            "create" => {
                mci.defer(ctx.http()).await?;
                return step_by_step_create_tournament(&ctx, &msg).await;
            }
            "start" => {
                mci.defer(ctx.http()).await?;
                todo!("Start a tournament");
            }
            _ => {
                unreachable!();
            }
        }
    }
    Ok(())
}

async fn step_by_step_config(ctx: &BotContext<'_>, msg: &ReplyHandle<'_>) -> Result<(), BotError> {
    async fn preset(ctx: &BotContext<'_>, msg: &ReplyHandle<'_>) -> Result<(), BotError> {
        let config = ctx.get_config().await?;
        if let Some(c) = config {
            let embed = CreateEmbed::default()
                .title("Configuration Already Set For This Server")
                .description(format!(
                    r#"Configuration already set for this server.\n
Marshal role: <@&{role}>,
Announcement channel: <#{ann}>,
Notification channel: <#{noti}>,
Log channel: <#{log}>.                
"#,
                    role = c.marshal_role_id,
                    ann = c.announcement_channel_id,
                    noti = c.notification_channel_id,
                    log = c.log_channel_id
                ))
                .color(Colour::GOLD);
            let components = vec![CreateActionRow::Buttons(vec![CreateButton::new("edit")
                .style(serenity::ButtonStyle::Primary)
                .label("Edit Configuration")])];
            let builder = CreateReply::default()
                .embed(embed)
                .components(components)
                .ephemeral(true);
            msg.edit(*ctx, builder).await?;
            while let Some(mci) = serenity::ComponentInteractionCollector::new(ctx)
                .author_id(ctx.author().id)
                .channel_id(ctx.channel_id())
                .filter(move |mci| mci.data.custom_id == "edit")
                .await
            {
                mci.defer(ctx.http()).await?;
                if mci.data.custom_id.as_str() == "edit" {
                    return Ok(());
                }
            }
        }
        Ok(())
    }
    preset(ctx, msg).await?;
    let marshal_role = select_role(
        ctx, 
        msg,
        "Select Marshal Role",
        "Please select the role that will be able to manage the tournament system.",
    ).await?;
    splash(ctx, msg).await?;
    let announcement_channel = select_channel(
        ctx,
        msg,
        "Select Announcement Channel",
        "Please select the channel where the bot will announce the progress of the tournament.",
    )
    .await?;
    splash(ctx, msg).await?;
    let notification_channel = select_channel(ctx, msg, "Select Notification Channel", "Please select the channel where the bot will send notifications to players about their progress and matches.").await?;
    splash(ctx, msg).await?;
    let log_channel = select_channel(
        ctx,
        msg,
        "Select Log Channel",
        "Please select the channel where the bot will log all the actions it takes.",
    )
    .await?;
    set_config(
        *ctx,
        marshal_role,
        announcement_channel,
        notification_channel,
        log_channel,
    )
    .await?;
    Ok(())
}

async fn step_by_step_create_tournament(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
) -> Result<(), BotError> {
    async fn input(ctx: &BotContext<'_>, msg: &ReplyHandle<'_>) -> Result<String, BotError> {
        #[derive(Debug, Modal)]
        #[name = "Tournament Name"]
        struct TournamentName {
            #[name = "Name the tournament here"]
            #[placeholder = ""]
            #[min_length = 4]
            #[max_length = 10]
            name: String,
        }
        let builder = {
            let components = vec![serenity::CreateActionRow::Buttons(vec![
                serenity::CreateButton::new("open_modal")
                    .label("Open modal")
                    .style(poise::serenity_prelude::ButtonStyle::Success),
            ])];

            poise::CreateReply::default()
                .content("Click the button below to open the modal")
                .components(components)
        };

        msg.edit(*ctx, builder).await?;

        while let Some(mci) = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
            .timeout(std::time::Duration::from_secs(120))
            .filter(move |mci| mci.data.custom_id == "open_modal")
            .await
        {
            let name = poise::execute_modal_on_component_interaction::<TournamentName>(
                ctx, mci, None, None,
            )
            .await?
            .ok_or(anyhow!("Modal interaction from <@{}> returned None. This may mean that the modal has timed out.", ctx.author().id.to_string()))?
            .name;
            return Ok(name);
        }  
        Err(anyhow!("No name entered").into())
    }
    let name = input(ctx, msg).await?;
    let role = select_role(
        ctx,
        msg,
        "Select Role",
        "Please select the role for the tournament.",
    ).await?;
    create_tournament(*ctx, name, role).await
}
/// Test for the match generation for new tournaments.
#[cfg(test)]
mod tests {
    use poise::serenity_prelude::Role;

    use super::generate_matches_new_tournament;
    use crate::database::{
        models::{Player, PlayerType, User},
        Database, PgDatabase,
    };

    fn create_dummy(sample: usize) -> Vec<User> {
        let mut users: Vec<User> = Vec::new();
        for index in 0..sample {
            let mut user = User::default();
            user.discord_id = index.to_string();
            user.player_tag = index.to_string();
            users.push(user);
        }
        users
    }

    #[tokio::test]
    async fn creates_two_matches() {
        let db = PgDatabase::connect().await.unwrap();
        const SAMPLE: usize = 4;
        let users: Vec<User> = create_dummy(SAMPLE);
        db.create_tournament("0", "test", -1, Role::default().id.to_string()).await.unwrap();

        println!("{:?}", users);

        for user in &users {
            db.create_user(user).await.unwrap();
            db.enter_tournament(-1, &user.discord_id).await.unwrap();
        }
        let players = users
            .iter()
            .map(|user| user.to_player())
            .collect::<Vec<Player>>();

        let matches = generate_matches_new_tournament(players, -1).await.unwrap();

        db.delete_tournament(-1).await.unwrap();

        println!("{:?}", matches);

        assert_eq!(matches.len(), 2);
        matches.iter().take(2).for_each(|game_match| {
            assert_eq!(game_match.player_1_type, PlayerType::Player);
            assert_eq!(game_match.player_2_type, PlayerType::Player);
        });
    }

    #[tokio::test]
    async fn creates_two_matches_with_one_bye() {
        const SAMPLE: usize = 3;
        let db = PgDatabase::connect().await.unwrap();

        let users = create_dummy(SAMPLE);
        db.create_tournament("0", "test", -2, Role::default().id.to_string()).await.unwrap();

        for user in &users {
            db.create_user(user).await.unwrap();
            db.enter_tournament(-1, &user.discord_id).await.unwrap();
        }
        let players = users
            .iter()
            .map(|user| user.to_player())
            .collect::<Vec<Player>>();
        let matches = generate_matches_new_tournament(players, -2).await.unwrap();

        db.delete_tournament(-2).await.unwrap();

        println!("{:?}", matches);

        assert_eq!(matches.len(), 2);
        assert!(matches[0].player_1_type == PlayerType::Player);
        assert!(matches[0].player_2_type == PlayerType::Player);
        assert!(matches[1].player_1_type == PlayerType::Player);
        assert!(matches[1].player_2_type == PlayerType::Dummy);
    }

    #[tokio::test]
    async fn creates_four_matches_with_two_byes() {
        const SAMPLE: usize = 6;
        let db = PgDatabase::connect().await.unwrap();
        let users = create_dummy(SAMPLE);
        db.create_tournament("0", "test", -3, Role::default().id.to_string()).await.unwrap();
        for user in &users {
            db.create_user(user).await.unwrap();
            db.enter_tournament(-3, &user.discord_id).await.unwrap();
        }
        let players = users
            .iter()
            .map(|user| user.to_player())
            .collect::<Vec<Player>>();
        let matches = generate_matches_new_tournament(players, -3).await.unwrap();

        db.delete_tournament(-3).await.unwrap();

        println!("{:?}", matches);

        assert_eq!(matches.len(), 4);
        assert!(matches[0].player_1_type == PlayerType::Player);
        assert!(matches[0].player_2_type == PlayerType::Player);
        assert!(matches[1].player_1_type == PlayerType::Player);
        assert!(matches[1].player_2_type == PlayerType::Player);
        assert!(matches[2].player_1_type == PlayerType::Player);
        assert!(matches[2].player_2_type == PlayerType::Dummy);
        assert!(matches[3].player_1_type == PlayerType::Player);
        assert!(matches[3].player_2_type == PlayerType::Dummy);
    }
}
