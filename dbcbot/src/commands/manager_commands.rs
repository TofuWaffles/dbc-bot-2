use super::CommandsContainer;
use crate::database::models::{Mode, Tournament};
use crate::database::{MatchDatabase, TournamentDatabase};
use crate::log::Log;
use crate::utils::discord::splash;
use crate::utils::error::CommonError::{self, *};
use crate::utils::shorthand::{BotComponent, BotContextExt};
use crate::{
    commands::checks::{is_config_set, is_manager},
    database::*,
    log, BotContext, BotData, BotError,
};
use anyhow::anyhow;
use models::{Match, Player, PlayerType, TournamentStatus};
use poise::serenity_prelude::{Channel, Mentionable, Role};
use poise::Modal;
use poise::{
    serenity_prelude::{self as serenity, Colour, CreateActionRow, CreateButton, CreateEmbed},
    CreateReply, ReplyHandle,
};
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
            force_result(),
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
    #[description = "This channel is set to receive mails from players!"]
    mail_channel: serenity::Channel,
) -> Result<(), BotError> {
    let msg = ctx
        .send(
            CreateReply::default()
                .content("Setting the configuration...")
                .ephemeral(true),
        )
        .await?;
    set_config(
        ctx,
        &msg,
        marshal_role,
        announcement_channel,
        mail_channel,
        log_channel,
    )
    .await
}

/// Creates a new tournament.
///
/// Tournament names do not have to be unique.
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
    #[description = "Announcement channel for the tournament"] announcement: serenity::Channel,
    #[description = "Notification channel for the tournament"] notification: serenity::Channel,
    #[description = "Number of wins required to win a match. Default: 2"] wins_required: Option<
        i32,
    >,
    #[description = "Role for the tournament"] role: Option<serenity::RoleId>,
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
///
/// The default number of wins required is 2.
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
            ctx.components()
                .prompt(
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

/// Set the configuration for the current server.
async fn set_config(
    ctx: BotContext<'_>,
    msg: &ReplyHandle<'_>,
    marshal_role: serenity::Role,
    announcement_channel: serenity::Channel,
    mail_channel: serenity::Channel,
    log_channel: serenity::Channel,
) -> Result<(), BotError> {
    let id = announcement_channel.id().to_string();
    let announcement_channel_id = match announcement_channel.guild() {
        Some(guild) => guild.id,
        None => {
            ctx.components()
                .prompt(
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
            ctx.log(log, None).await?;
            error!("Invalid announcement channel entered by {}", ctx.author());
            return Err(ChannelNotExists(id).into());
        }
    };
    let id = log_channel.id().to_string();
    let log_channel_id = match log_channel.guild() {
        Some(guild) => guild.id,
        None => {
            ctx.components()
                .prompt(
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
            ctx.log(log, None).await?;
            error!("Invalid log channel entered by {}", ctx.author());
            return Err(ChannelNotExists(id).into());
        }
    };

    let mail_channel_id = match mail_channel.guild() {
        Some(guild) => guild.id,
        None => {
            ctx.components()
                .prompt(
                    msg,
                    CreateEmbed::new()
                        .title("Invalid mail channel")
                        .description(
                            "Please enter a valid server channel to set this mail channel.",
                        )
                        .color(Colour::RED),
                    None,
                )
                .await?;
            let log = ctx.build_log(
                "MANAGER CONFIGURATION SET FAILED!",
                format!("Invalid mail channel selected: {}", id),
                log::State::FAILURE,
                log::Model::MARSHAL,
            );
            ctx.log(log, None).await?;
            error!("Invalid mail channel entered by {}", ctx.author());
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
            mail_channel_id.as_ref(),
            announcement_channel_id.as_ref(),
        )
        .await?;
    ctx.components()
        .prompt(
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
    ctx.log(log, None).await?;

    Ok(())
}

/// Create a new tournament.
async fn create_tournament(
    ctx: BotContext<'_>,
    msg: &ReplyHandle<'_>,
    name: String,
    mode: Mode,
    role: Option<serenity::RoleId>,
    announcement_channel: serenity::Channel,
    notification_channel: serenity::Channel,
    wins_required: i32,
) -> Result<(), BotError> {
    let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;
    let new_tournament_id = ctx
        .data()
        .database
        .create_tournament(
            &guild_id,
            &name,
            &mode,
            None,
            &role,
            &announcement_channel.id(),
            &notification_channel.id(),
            wins_required,
        )
        .await?;
    ctx.components()
        .prompt(
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
        ("Role", role.unwrap().mention().to_string(), true),
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
    ctx.log(log, None).await?;
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
            ctx.components().prompt(
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
        .get_tournament_players(tournament.tournament_id, None)
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
            .create_match(
                tournament_id,
                bracket.round()?,
                bracket.sequence()?,
                bracket.winner,
                bracket.score.into(),
            )
            .await?;
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
    ctx.components().prompt(
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
    ctx.log(log, None).await?;

    Ok(())
}

/// Marshal menu command.
///
/// Allow access to all marshal commands using an easy-to-use Graphical User Interface.
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
            let marshal_role = c.marshal(ctx).await?;
            let announcement_channel = c.announcement_channel(ctx).await?;
            let log_channel = c.log_channel(ctx).await?;
            let mail_channel = c.mail_channel(ctx).await?;

            let embed = CreateEmbed::default()
                .title("Server Configuration")
                .description("The following configuration is currently set for this server.")
                .fields(vec![
                    ("Marshal Role", marshal_role.mention().to_string(), true),
                    (
                        "Announcement Channel",
                        announcement_channel.mention().to_string(),
                        true,
                    ),
                    ("Mail Channel", mail_channel.mention().to_string(), true),
                    ("Log Channel", log_channel.mention().to_string(), true),
                ])
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
    let embed = |m: &Role, a: &Channel, mail: &Channel, l: &Channel| {
        CreateEmbed::default()
            .title("Configuration Confirmation")
            .description("Please confirm the following configuration:")
            .fields(vec![
                ("Marshal Role", m.mention().to_string(), true),
                ("Announcement Channel", a.mention().to_string(), true),
                ("Mail Channel", mail.mention().to_string(), true),
                ("Log Channel", l.mention().to_string(), true),
            ])
            .color(Colour::GOLD)
    };
    preset(ctx, msg).await?;
    let (m, a, mail, l) = loop {
        let membed = CreateEmbed::default()
            .title("Select Marshal Role")
            .description(
                "Please select the role that will be able to manage the tournament system.",
            );
        let marshal_role = ctx.components().select_role(msg, membed).await?;
        splash(ctx, msg).await?;
        let aembed = CreateEmbed::default()
            .title("Select Announcement Channel")
            .description(
            "Please select the channel where the bot will announce the progress of the tournament.",
        );

        let announcement_channel = ctx.components().select_channel(msg, aembed).await?;
        splash(ctx, msg).await?;
        let lembed = CreateEmbed::default()
            .title("Select Log Channel")
            .description(
                "Please select the channel where the bot will log all the actions it takes.",
            );
        let log_channel = ctx.components().select_channel(msg, lembed).await?;
        splash(ctx, msg).await?;
        let mail_embed = CreateEmbed::default()
            .title("Select Mail Channel")
            .description(
                "Please select the channel where the bot will send mails to players about their progress and matches.",
            );
        let mail_channel = ctx.components().select_channel(msg, mail_embed).await?;
        if ctx
            .components()
            .confirmation(
                msg,
                embed(
                    &marshal_role,
                    &announcement_channel,
                    &mail_channel,
                    &log_channel,
                ),
            )
            .await?
        {
            break (
                marshal_role,
                announcement_channel,
                mail_channel,
                log_channel,
            );
        }
    };

    set_config(*ctx, msg, m, a, mail, l).await?;
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
        let modal = ctx
            .components()
            .modal::<TournamentName>(msg, m_embed.clone())
            .await?;
        let mode = ctx
            .components()
            .select_options::<Mode>(
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
        let announcement_channel = ctx.components().select_channel(msg, aembed).await?;
        splash(ctx, msg).await?;
        let nembed = CreateEmbed::default()
            .title("Select Notification Channel")
            .description("Please select the channel where the bot will send notifications to players about their progress and matches.");
        let notification_channel = ctx.components().select_channel(msg, nembed).await?;
        let rembed = CreateEmbed::default()
            .title("Select Role")
            .description("Please select the role for the tournament.");
        let role = ctx.components().select_role(msg, rembed).await?;
        if ctx
            .components()
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
        r.id.into(),
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
    let id = ctx
        .components()
        .select_options::<Tournament>(
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

    let mut matches_count = 1 << (rounds_count - 1);

    let mut matches = Vec::new();
    let mut round = 1;

    while matches_count != 0 {
        // We use range from 1 to match_count inclusive for convenience
        // Because it makes it easier to calculate the parent brackets (i << 1 - 1, i << 1)
        for i in 1..=matches_count {
            let mut bracket = Match::new(tournament_id, round, i, Vec::new(), "0-0");

            if round == 1 {
                // Not guaranteed to have a player, this would be a bye round if there is no player
                //
                // We use a greedy approach by determining a threshold using the number of slots
                // still available
                //
                // The # of slots is the number of (total # of matches - current # of matches) * 2
                // If the number of players remaining is above this threshold, we put both players
                // in.
                if tournament_players.len() >= (matches_count as usize - matches.len()) << 1 {
                    bracket.match_players.push(tournament_players.pop().ok_or(anyhow!("Error generation matches for new tournament: the match count ({}), does not match the number of players ({})", matches_count, tournament_players.len()))?.into());
                }
                // Guaranteed to have a player
                bracket.match_players.push(tournament_players.pop().ok_or(anyhow!("Error generation matches for new tournament: the match count ({}), does not match the number of players ({})", matches_count, tournament_players.len()))?.into());
                if bracket.match_players.len() == 1 {
                    bracket.winner = bracket.match_players[0].discord_id.to_string().into();
                    bracket.score = "BYE".to_string();
                }
            } else if round == 2 {
                // For the second round, we check for any bye matches in the previous round
                let left: &Match = &matches[((i << 1) - 2) as usize];
                let right: &Match = &matches[((i << 1) - 1) as usize];
                if left.winner.is_some() {
                    bracket.match_players.push(left.match_players[0].clone());
                }
                if right.winner.is_some() {
                    bracket.match_players.push(right.match_players[0].clone());
                }
            }
            // Bye rounds don't make it past the second round
            // The rest of the matches past the second round are guaranteed to be empty

            matches.push(bracket);
        }
        matches_count >>= 1;
        round += 1;
    }

    Ok(matches)
}

/// Force the result of a match by manually setting the winner even if the match already happened.
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    check = "is_manager",
    check = "is_config_set",
    rename = "force_result"
)]
async fn force_result(
    ctx: BotContext<'_>,
    #[description = "Match ID. Format: <tournament_id>-<round>-<sequence>"] match_id: String,
    #[description = "The winner of the match"] winner: serenity::User,
    #[description = "The player who was eliminated"] eliminated: serenity::User,
    #[description = "The score of the match. Default: `WON-LOST`"] score: Option<String>,
) -> Result<(), BotError> {
    async fn output(ctx: &BotContext<'_>, embed: CreateEmbed) -> Result<(), BotError> {
        let reply = CreateReply::default()
            .embed(embed)
            .ephemeral(true)
            .reply(true);
        ctx.send(reply).await?;
        Ok(())
    }

    if !Match::is_valid_id(match_id.as_str()) {
        let embed = CreateEmbed::default()
            .title("Invalid match ID")
            .description("The match ID provided is invalid.")
            .color(Colour::RED);
        return output(&ctx, embed).await;
    };
    if winner.id == eliminated.id {
        let embed = CreateEmbed::default()
            .title("Invalid match result")
            .description("The winner and the eliminated player cannot be the same.")
            .color(Colour::RED);
        return output(&ctx, embed).await;
    }
    let score = match score {
        Some(scr) => {
            if !Match::is_valid_score(scr.as_str()) {
                let embed = CreateEmbed::default()
                    .title("Invalid score")
                    .description(
                        "The score provided is invalid. Accepted format: `2-1`, `2-0`, `text-text`",
                    )
                    .color(Colour::RED);
                return output(&ctx, embed).await;
            }
            scr
        }
        None => "WON-LOST".to_string(),
    };
    let Some(mut game_match) = ctx
        .data()
        .database
        .get_match_by_id(match_id.as_str())
        .await?
    else {
        let embed = CreateEmbed::default()
            .title("Match not found")
            .description("The match with the given ID was not found.")
            .color(Colour::RED);
        return output(&ctx, embed).await;
    };
    if game_match.get_player(&winner.id.to_string()).is_err() {
        let embed = CreateEmbed::default()
            .title("Invalid winner")
            .description("The winner provided is not a part of the match.")
            .color(Colour::RED);
        return output(&ctx, embed).await;
    }
    if game_match.get_player(&eliminated.id.to_string()).is_err() {
        let embed = CreateEmbed::default()
            .title("Invalid eliminated player")
            .description("The eliminated player provided is not a part of the match.")
            .color(Colour::RED);
        return output(&ctx, embed).await;
    }
    match game_match.winner(&ctx).await? {
        Some(current_winner) => {
            if current_winner == winner {
                let embed = CreateEmbed::default()
                    .title("Winner already set")
                    .description("The winner for this match has already been set.")
                    .color(Colour::RED);
                return output(&ctx, embed).await;
            }
            game_match.winner = Some(winner.id.to_string());
            game_match.score = score;
            ctx.data()
                .database
                .update_match_result(&ctx, game_match)
                .await?;
        }
        None => {
            let msg = ctx
                .send(
                    CreateReply::default()
                        .embed(CreateEmbed::new().description("Loading"))
                        .ephemeral(true),
                )
                .await?;
            let embed = CreateEmbed::default()
                .title("The match has not started yet.")
                .description("It is recommended to use /disqualify instead. Do you want to continue?\n -# Note: The difference between this command and /disqualify is at score display, if score is not provided, the score will be displayed as `WON-LOST`.")
                .color(Colour::RED);

            let decision = ctx.components().confirmation(&msg, embed).await?;
            if !decision {
                let embed = CreateEmbed::default()
                    .title("Operation cancelled")
                    .description("The operation has been cancelled.")
                    .color(Colour::RED);
                return output(&ctx, embed).await;
            }
            game_match.winner = Some(winner.id.to_string());
            game_match.score = score;
            ctx.data()
                .database
                .update_match_result(&ctx, game_match)
                .await?;
        }
    }

    Ok(())
}

/// Test for the match generation for new tournaments.
#[cfg(test)]
mod tests {
    use super::{
        generate_matches_new_tournament,
        models::{Match, Player},
    };

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

    fn check_empty(bracket: &Match) -> bool {
        if bracket.match_players.get(0).is_none() && bracket.match_players.get(1).is_none() {
            return true;
        }
        false
    }

    fn check_full(bracket: &Match) -> bool {
        if bracket.match_players.get(0).is_some() && bracket.match_players.get(1).is_some() {
            return true;
        }
        false
    }

    fn check_bye(bracket: &Match) -> bool {
        if bracket.match_players.get(0).is_some() && bracket.match_players.get(1).is_none() {
            return true;
        }
        false
    }

    #[test]
    fn creates_two_rounds() {
        const USERCOUNT: i32 = 4;
        let players: Vec<Player> = create_dummies(USERCOUNT);

        println!("{:?}", players);

        let matches = generate_matches_new_tournament(players, -1).unwrap();

        assert_eq!(matches.len(), 3);
        assert!(check_full(&matches[0]));
        assert!(check_full(&matches[1]));
        assert!(check_empty(&matches[2]));
    }

    #[test]
    fn creates_two_rounds_with_one_bye() {
        const USERCOUNT: i32 = 3;
        let users = create_dummies(USERCOUNT);

        println!("{:?}", users);

        let matches = generate_matches_new_tournament(users, -1).unwrap();

        println!("{:?}", matches);

        assert_eq!(matches.len(), 3);
        assert!(check_bye(&matches[0]));
        assert!(matches[0].winner.is_some());
        assert!(check_full(&matches[1]));
        assert!(check_bye(&matches[2]));
        assert!(matches[2].winner.is_none());
    }

    #[test]
    fn creates_three_rounds() {
        const USERCOUNT: i32 = 8;
        let users = create_dummies(USERCOUNT);

        println!("{:?}", users);

        let matches = generate_matches_new_tournament(users, -1).unwrap();

        assert_eq!(matches.len(), 7);

        matches.iter().enumerate().for_each(|(i, m)| {
            match i {
                0..5 => check_full(m),
                _ => check_empty(m),
            };
        });
    }

    #[test]
    fn creates_three_rounds_with_two_byes() {
        const USERCOUNT: i32 = 6;
        let users = create_dummies(USERCOUNT);

        let matches = generate_matches_new_tournament(users, -1).unwrap();

        println!("{:?}", matches);

        assert_eq!(matches.len(), 7);

        assert!(check_bye(&matches[0]));
        assert!(matches[0].winner.is_some());
        assert!(check_bye(&matches[1]));
        assert!(matches[1].winner.is_some());
        assert!(check_full(&matches[2]));
        assert!(check_full(&matches[3]));
        assert!(check_full(&matches[4]));
        assert!(check_empty(&matches[5]));
        assert!(check_empty(&matches[6]));
    }

    #[test]
    fn create_four_rounds() {
        const USERCOUNT: i32 = 16;
        let users = create_dummies(USERCOUNT);

        let matches = generate_matches_new_tournament(users, -1).unwrap();

        println!("{:?}", matches);

        assert_eq!(matches.len(), 15);

        matches.iter().enumerate().for_each(|(i, m)| {
            match i {
                0..8 => assert!(check_full(m)),
                _ => assert!(check_empty(m)),
            };
        });
    }

    #[test]

    fn create_four_rounds_with_seven_byes() {
        const USERCOUNT: i32 = 9;

        let users = create_dummies(USERCOUNT);

        let matches = generate_matches_new_tournament(users, -1).unwrap();

        println!("{:?}", matches);

        assert_eq!(matches.len(), 15);

        matches.iter().enumerate().for_each(|(i, m)| match i {
            0..7 => {
                assert!(check_bye(m));
                assert!(m.winner.is_some());
            }
            7 => {
                assert!(check_full(m));
                assert!(m.winner.is_none());
            }
            8..11 => {
                assert!(check_full(m));
                assert!(m.winner.is_none());
            }
            11 => {
                assert!(check_bye(m));
                assert!(m.winner.is_none());
            }
            _ => assert!(check_empty(m)),
        });
    }
}
