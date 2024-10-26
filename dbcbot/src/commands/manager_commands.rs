use super::CommandsContainer;
use crate::database::models::{Mode, Tournament};
use crate::database::{MatchDatabase, TournamentDatabase};
use crate::log::Log;
use crate::utils::discord::{modal, select_channel, select_options, select_role, splash};
use crate::utils::error::CommonError::{self, *};
use crate::utils::shorthand::BotContextExt;
use crate::{
    commands::checks::{is_config_set, is_manager},
    database::*,
    log, BotContext, BotData, BotError,
};
use anyhow::anyhow;
use models::{Match, MatchPlayer, Player, PlayerType, TournamentStatus};
use poise::serenity_prelude::{Channel, Mentionable, Role, UserId};
use poise::Modal;
use poise::{
    serenity_prelude::{self as serenity, Colour, CreateActionRow, CreateButton, CreateEmbed},
    CreateReply, ReplyHandle,
};
use std::str::FromStr;
use tracing::{error, info, instrument};
const DEFAULT_WIN_REQUIRED: i32 = 2;
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
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    check = "is_manager",
    rename = "set_config"
)]
#[instrument]
async fn set_config_slash(
    ctx: BotContext<'_>,
    #[description = "This role can access tournament monitor commands!"]
    marshal_role: serenity::Role,
    #[description = "This channel is set for general announcement for the tournament!"]
    announcement_channel: serenity::Channel,
    #[description = "This channel logs activities"] log_channel: serenity::Channel,
) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default()
                .content("Setting the configuration...")
                .ephemeral(true),
        )
        .await?;
    set_config(ctx, &msg, marshal_role, announcement_channel, log_channel).await
}

/// Create a new tournament.
///
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    check = "is_manager",
    rename = "create_tournament"
)]
#[instrument]
async fn create_tournament_slash(
    ctx: BotContext<'_>,
    #[description = "Tournament name"] name: String,
    #[description = "Mode for the tournament"] mode: Mode,
    #[description = "Role for the tournament"] role: serenity::Role,
    #[description = "Announcement channel for the tournament"] announcement: serenity::Channel,
    #[description = "Notification channel for the tournament"] notification: serenity::Channel,
    #[description = "Number of wins required to win a match. Default: 2"] wins_required: Option<
        i32,
    >,
) -> Result<(), BotError> {
    let wins_required = wins_required.unwrap_or(2).max(1);
    let msg = ctx
        .send(
            CreateReply::default()
                .content("Creating a new tournament...")
                .ephemeral(true),
        )
        .await?;
    create_tournament(
        ctx,
        &msg,
        name,
        mode,
        role,
        announcement,
        notification,
        wins_required,
    )
    .await
}

/// Start a tournament.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    check = "is_manager",
    check = "is_config_set",
    rename = "start_tournament"
)]
#[instrument]
async fn start_tournament_slash(
    ctx: BotContext<'_>,
    tournament_id: i32,
    win_required: Option<i32>,
) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default()
                .content("Starting the tournament...")
                .ephemeral(true),
        )
        .await?;
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;
    let tournament = match ctx
        .data()
        .database
        .get_tournament(&guild_id, tournament_id)
        .await?
    {
        Some(tournament) => tournament,
        None => {
            ctx.prompt(
                &msg,
                CreateEmbed::default()
                    .title("Tournament not found")
                    .description("The tournament with the given ID was not found.")
                    .color(Colour::RED),
                None,
            )
            .await?;
            return Ok(());
        }
    };
    start_tournament(ctx, &msg, tournament).await
}

async fn set_config(
    ctx: BotContext<'_>,
    msg: &ReplyHandle<'_>,
    marshal_role: serenity::Role,
    announcement_channel: serenity::Channel,
    log_channel: serenity::Channel,
) -> Result<(), BotError> {
    let id = announcement_channel.id().to_string();
    let announcement_channel_id = match announcement_channel.guild() {
        Some(guild) => guild.id,
        None => {
            ctx.prompt(
                msg,
                CreateEmbed::new()
                    .title("Invalid announcement channel")
                    .description(
                        "Please enter a valid server channel to set this announcement channel.",
                    )
                    .color(Colour::RED),
                None,
            )
            .await?;
            let log = ctx.build_log(
                "MANAGER CONFIGURATION SET FAILED!",
                format!("Invalid announcement channel selected: {}", id),
                log::State::FAILURE,
                log::Model::MARSHAL,
            );
            ctx.log(log).await?;
            error!("Invalid announcement channel entered by {}", ctx.author());
            return Err(ChannelNotExists(id).into());
        }
    };
    let id = log_channel.id().to_string();
    let log_channel_id = match log_channel.guild() {
        Some(guild) => guild.id,
        None => {
            ctx.prompt(
                msg,
                CreateEmbed::new()
                    .title("Invalid log channel")
                    .description("Please enter a valid server channel to set this log channel.")
                    .color(Colour::RED),
                None,
            )
            .await?;
            let log = ctx.build_log(
                "MANAGER CONFIGURATION SET FAILED!",
                format!("Invalid log channel selected: {}", id),
                log::State::FAILURE,
                log::Model::MARSHAL,
            );
            ctx.log(log).await?;
            error!("Invalid log channel entered by {}", ctx.author());
            return Err(ChannelNotExists(id).into());
        }
    };

    let marshal_role_id = marshal_role.id;

    ctx.data()
        .database
        .set_config(
            ctx.guild_id().ok_or(NotInAGuild)?.as_ref(),
            marshal_role_id.as_ref(),
            log_channel_id.as_ref(),
            announcement_channel_id.as_ref(),
        )
        .await?;
    ctx.prompt(
        msg,
        CreateEmbed::new()
            .title("Configuration set successfully!")
            .description("Run this command again if you want to change the configuration.")
            .color(Colour::DARK_GREEN),
        None,
    )
    .await?;

    info!(
        "Set the configuration for guild {}",
        ctx.guild_id().unwrap().to_string()
    );
    let log = ctx.build_log(
        "General configuration set!",
        "The setting is set successfully!",
        log::State::SUCCESS,
        log::Model::GUILD,
    );
    ctx.log(log).await?;

    Ok(())
}

/// Create a new tournament.
async fn create_tournament(
    ctx: BotContext<'_>,
    msg: &ReplyHandle<'_>,
    name: String,
    mode: Mode,
    role: serenity::Role,
    announcement_channel: serenity::Channel,
    notification_channel: serenity::Channel,
    wins_required: i32,
) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;
    let role_id = role.id;
    let new_tournament_id = ctx
        .data()
        .database
        .create_tournament(
            &guild_id,
            &name,
            &mode,
            None,
            &role_id,
            &announcement_channel.id(),
            &notification_channel.id(),
            wins_required,
        )
        .await?;
    ctx.prompt(
        msg,
        CreateEmbed::new()
            .title("Successfully create a new tournament")
            .description(format!("Tournament id: {}", new_tournament_id)),
        None,
    )
    .await?;
    let fields = vec![
        ("Tournament ID", new_tournament_id.to_string(), true),
        ("Tournament name", name, true),
        ("Mode", mode.to_string(), true),
        ("Role", role.mention().to_string(), true),
        (
            "Announcement channel",
            announcement_channel.mention().to_string(),
            true,
        ),
        (
            "Notification channel",
            notification_channel.mention().to_string(),
            true,
        ),
        ("Wins required", wins_required.to_string(), true),
    ];
    let log = ctx
        .build_log(
            "Tournament created successfully!",
            "",
            log::State::SUCCESS,
            log::Model::TOURNAMENT,
        )
        .fields(fields);
    ctx.log(log).await?;
    info!(
        "Created tournament {} for guild {}",
        new_tournament_id, guild_id
    );

    Ok(())
}

async fn start_tournament(
    ctx: BotContext<'_>,
    msg: &ReplyHandle<'_>,
    tournament: Tournament,
) -> Result<(), BotError> {
    let tournament_id = tournament.tournament_id;
    match tournament.status {
        TournamentStatus::Pending => (),
        _ => {
            ctx.prompt(
                msg,
                CreateEmbed::default()
                    .title("Tournament already started or ended")
                    .description(
                        "The tournament has already started or ended. You can't start it again.",
                    )
                    .color(Colour::RED),
                None,
            )
            .await?;
            return Ok(());
        }
    }

    let tournament_players = ctx
        .data()
        .database
        .get_tournament_players(tournament.tournament_id)
        .await?;

    if tournament_players.len() < 2 {
        ctx.send(
            CreateReply::default()
                .content(format!(
                    "There are not enough players to start the tournament with ID {}.",
                    tournament.tournament_id
                ))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let rounds_count = (tournament_players.len() as f64).log2().ceil() as i32;

    let matches = generate_matches_new_tournament(tournament_players, tournament_id)?;

    let matches_count = matches.len();

    for bracket in matches {
        ctx.data()
            .database
            .create_match(tournament_id, bracket.round()?, bracket.sequence()?)
            .await?;
        // Bye round. Immediately set the winner
        if bracket.match_players.len() == 1 {
            ctx.data()
                .database
                .set_winner(
                    &bracket.match_id,
                    &UserId::from_str(bracket.match_players[0].discord_id.as_str())?,
                    "bye",
                )
                .await?;
        }
        for player in bracket.match_players {
            let player_id = player.user_id()?;
            ctx.data()
                .database
                .enter_match(&bracket.match_id, &player_id, PlayerType::Player)
                .await?;
        }
    }

    ctx.data()
        .database
        .set_current_round(tournament_id, 1)
        .await?;
    ctx.data()
        .database
        .set_tournament_status(tournament_id, TournamentStatus::Started)
        .await?;

    ctx.data()
        .database
        .set_rounds(tournament_id, rounds_count)
        .await?;
    ctx.prompt(
        msg,
        CreateEmbed::default()
            .title("Tournament started!")
            .description(format!(
                "Successfully started tournament with ID {}.\n\nTotal number of matches in the first round (including byes): {}",
                tournament_id, matches_count
            ))
            .color(Colour::DARK_GREEN),
        None,
    ).await?;
    let fields = vec![
        ("Tournament ID", tournament_id.to_string(), true),
        ("Tournament name", tournament.name, true),
        ("Rounds", rounds_count.to_string(), true),
        ("Number of matches", matches_count.to_string(), true),
        ("Started by", ctx.author().name.clone(), true),
    ];
    let log = ctx
        .build_log(
            "Tournament started successfully!",
            "",
            log::State::SUCCESS,
            log::Model::TOURNAMENT,
        )
        .fields(fields);
    ctx.log(log).await?;

    Ok(())
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
                return step_by_step_start_tournament(&ctx, &msg).await;
            }
            _ => {
                continue;
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
Log channel: <#{log}>.                
"#,
                    role = c.marshal_role_id,
                    ann = c.announcement_channel_id,
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
    let embed = |m: &Role, a: &Channel, l: &Channel| {
        CreateEmbed::default()
            .title("Configuration Confirmation")
            .description(format!(
                r#"Please confirm the following configuration:
Marshal role: <@&{role}>,
Announcement channel: <#{ann}>,
Log channel: <#{log}>.
"#,
                role = m.id.get(),
                ann = a.id().get(),
                log = l.id().get()
            ))
            .color(Colour::GOLD)
    };
    preset(ctx, msg).await?;
    let (m, a, l) = loop {
        let membed = CreateEmbed::default()
            .title("Select Marshal Role")
            .description(
                "Please select the role that will be able to manage the tournament system.",
            );
        let marshal_role = select_role(ctx, msg, membed).await?;
        splash(ctx, msg).await?;
        let aembed = CreateEmbed::default()
            .title("Select Announcement Channel")
            .description(
            "Please select the channel where the bot will announce the progress of the tournament.",
        );

        let announcement_channel = select_channel(ctx, msg, aembed).await?;
        splash(ctx, msg).await?;
        let lembed = CreateEmbed::default()
            .title("Select Log Channel")
            .description(
                "Please select the channel where the bot will log all the actions it takes.",
            );
        let log_channel = select_channel(ctx, msg, lembed).await?;
        if ctx
            .confirmation(
                msg,
                embed(&marshal_role, &announcement_channel, &log_channel),
            )
            .await?
        {
            break (marshal_role, announcement_channel, log_channel);
        }
    };

    set_config(*ctx, msg, m, a, l).await?;
    Ok(())
}

async fn step_by_step_create_tournament(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
) -> Result<(), BotError> {
    #[derive(Debug, Modal)]
    #[name = "Tournament Name"]
    struct TournamentName {
        #[name = "Name the tournament here"]
        #[placeholder = ""]
        #[min_length = 4]
        #[max_length = 10]
        name: String,

        #[name = "Minimum wins required to win a match"]
        #[placeholder = "Write the number of wins required to win a match here or leave it blank for 2!"]
        wins_required: Option<String>,
    }
    let embed = |m: &TournamentName, r: &Role, a: &Channel, n: &Channel| {
        CreateEmbed::default()
            .title("Tournament Confirmation")
            .description("Please confirm the following tournament:")
            .fields(vec![
                ("Tournament name", m.name.clone(), true),
                ("Role", r.mention().to_string(), true),
                ("Announcement channel", a.mention().to_string(), true),
                ("Notification channel", n.mention().to_string(), true),
                (
                    "Wins required",
                    m.wins_required
                        .clone()
                        .unwrap_or(DEFAULT_WIN_REQUIRED.to_string()),
                    true,
                ),
            ])
            .color(Colour::GOLD)
    };
    let (m, mode, a, n, r) = loop {
        let m_embed = CreateEmbed::new()
            .title("Creating a new tournament")
            .description("Please provide the name of the tournament.");
        let modal = modal::<TournamentName>(ctx, msg, m_embed.clone()).await?;
        let mode = select_options::<Mode>(
            ctx,
            msg,
            CreateEmbed::default()
                .title("Select Mode")
                .description("Please select the mode for the tournament."),
            None,
            &Mode::all(),
        )
        .await?;
        splash(ctx, msg).await?;
        let aembed = CreateEmbed::default()
            .title("Select Announcement Channel")
            .description(
            "Please select the channel where the bot will announce the progress of the tournament.",
        );
        let announcement_channel = select_channel(ctx, msg, aembed).await?;
        splash(ctx, msg).await?;
        let nembed = CreateEmbed::default()
            .title("Select Notification Channel")
            .description("Please select the channel where the bot will send notifications to players about their progress and matches.");
        let notification_channel = select_channel(ctx, msg, nembed).await?;
        let rembed = CreateEmbed::default()
            .title("Select Role")
            .description("Please select the role for the tournament.");
        let role = select_role(ctx, msg, rembed).await?;
        if ctx
            .confirmation(
                msg,
                embed(&modal, &role, &announcement_channel, &notification_channel),
            )
            .await?
        {
            break (
                modal,
                mode,
                announcement_channel,
                notification_channel,
                role,
            );
        }
    };
    let name = m.name;

    let wins_required = m
        .wins_required
        .map(|x| x.parse::<i32>().unwrap_or(DEFAULT_WIN_REQUIRED).max(1))
        .unwrap_or(DEFAULT_WIN_REQUIRED);
    create_tournament(
        *ctx,
        msg,
        name,
        Mode::from_string(mode),
        r,
        a,
        n,
        wins_required,
    )
    .await
}

async fn step_by_step_start_tournament(
    ctx: &BotContext<'_>,
    msg: &ReplyHandle<'_>,
) -> Result<(), BotError> {
    #[derive(Debug, Modal)]
    #[name = "More settings"]
    struct More {
        #[name = "Number of wins required to win a match"]
        #[placeholder = "Write the number of wins required to win a match here or leave it blank for 3!"]
        wins_required: Option<String>,
    }
    let guild_id = ctx.guild_id().ok_or(anyhow!("No guild id found"))?;
    let tournaments = ctx.data().database.get_all_tournaments(&guild_id).await?;
    let id = select_options::<Tournament>(
        ctx,
        msg,
        CreateEmbed::default()
            .title("Start Tournament")
            .description("Select a tournament you want to start"),
        None,
        &tournaments,
    )
    .await?;
    let tournament = tournaments
        .into_iter()
        .find(|t| t.tournament_id.to_string() == id)
        .ok_or(CommonError::TournamentNotExists(id))?;
    start_tournament(*ctx, msg, tournament).await
}

/// Contains the logic for generating matches for a newly started tournament.
///
/// Returns a Vector of tuples.
/// Each tuple contains a Match and a Vector of Users.
fn generate_matches_new_tournament(
    mut tournament_players: Vec<Player>,
    tournament_id: i32,
) -> Result<Vec<Match>, BotError> {
    let rounds_count = (tournament_players.len() as f64).log2().ceil() as u32;

    let match_count = 2_u32.pow(rounds_count - 1);

    let mut matches = Vec::new();

    for i in 0..match_count {
        let mut players: Vec<MatchPlayer> = Vec::new();
        // Not guaranteed to have a player, this would be a bye round if there is no player
        if (match_count as usize) <= tournament_players.len() {
            players.push(tournament_players.pop().ok_or(anyhow!("Error generation matches for new tournament: the match count ({}), does not match the number of players ({})", match_count, tournament_players.len()))?.into());
        }
        // Guaranteed to have a player
        players.push(tournament_players.pop().ok_or(anyhow!("Error generation matches for new tournament: the match count ({}), does not match the number of players ({})", match_count, tournament_players.len()))?.into());

        matches.push(Match::new(tournament_id, 1, (i + 1) as i32, players, "0-0"));
    }

    Ok(matches)
}

/// Test for the match generation for new tournaments.
#[cfg(test)]
mod tests {
    use super::{generate_matches_new_tournament, models::Player};

    fn create_dummies(count: i32) -> Vec<Player> {
        let mut users: Vec<Player> = Vec::with_capacity(count as usize);
        for index in 0..count {
            let mut user = Player::default();
            user.discord_id = index.to_string();
            user.player_tag = index.to_string();
            users.push(user);
        }
        users
    }

    #[test]
    fn creates_two_matches() {
        const USERCOUNT: i32 = 4;
        let users: Vec<Player> = create_dummies(USERCOUNT);

        println!("{:?}", users);

        let matches = generate_matches_new_tournament(users, -1).unwrap();

        assert_eq!(matches.len(), 2);
        matches.iter().enumerate().for_each(|(i, game_match)| {
            assert_eq!(game_match.sequence().unwrap(), i as i32 + 1);
            assert!(game_match.match_players.get(0).is_some());
            assert!(game_match.match_players.get(1).is_some());
        });
    }

    #[test]
    fn creates_two_matches_with_one_bye() {
        const USERCOUNT: i32 = 3;
        let users = create_dummies(USERCOUNT);

        println!("{:?}", users);

        let matches = generate_matches_new_tournament(users, -2).unwrap();

        println!("{:?}", matches);

        assert_eq!(matches.len(), 2);
        assert!(matches[0].match_players.get(0).is_some());
        assert!(matches[0].match_players.get(1).is_some());
        assert!(matches[1].match_players.get(0).is_some());
        assert!(matches[1].match_players.get(1).is_none());
    }

    #[test]
    fn creates_four_matches_with_two_byes() {
        const USERCOUNT: i32 = 6;
        let users = create_dummies(USERCOUNT);

        let matches = generate_matches_new_tournament(users, -3).unwrap();

        println!("{:?}", matches);

        assert_eq!(matches.len(), 4);

        matches.iter().enumerate().for_each(|(i, gm)| match i {
            2.. => {
                assert!(gm.match_players.get(0).is_some());
                assert!(gm.match_players.get(1).is_none());
            }
            _ => {
                assert!(gm.match_players.get(0).is_some());
                assert!(gm.match_players.get(1).is_some());
            }
        });
    }
}
