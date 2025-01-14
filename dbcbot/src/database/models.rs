use super::{BattleDatabase, ConfigDatabase, Database, MatchDatabase, TournamentDatabase};
use crate::api::official_brawl_stars::TeamPlayer;
use crate::utils::discord::DiscordTrait;
use crate::utils::error::CommonError::*;
use crate::utils::shorthand::BotContextExt;
use crate::{api::official_brawl_stars::Brawler, BotContext, BotError};
use poise::serenity_prelude::{Channel, GuildChannel, Role, User, UserId};
use serde::{Deserialize, Serialize};
use std::vec;
use strum::{Display, EnumIter, IntoEnumIterator};

/// Types that can be selected by the user in a dropdown menu.
pub trait Selectable {
    /// The string from the selection that the user sees.
    fn label(&self) -> String;
    /// The ID value used to uniquely identify the selection.
    fn identifier(&self) -> String;
}
/// The manager role configuration for a guild within the database.
#[derive(Serialize, Deserialize)]
pub struct ManagerRoleConfig {
    pub guild_id: String,
    pub manager_role_id: String,
}

impl DiscordTrait for ManagerRoleConfig {}
#[allow(dead_code)]
impl ManagerRoleConfig {
    #[inline]
    pub async fn to_manager(&self, ctx: &BotContext<'_>) -> Result<Role, BotError> {
        Self::to_role(ctx, &self.manager_role_id).await
    }
}

/// The configuration for a guild within the database.
#[derive(Serialize, Deserialize)]
pub struct GuildConfig {
    pub guild_id: String,
    pub marshal_role_id: String,
    pub log_channel_id: String,
    pub mail_channel_id: String,
    pub announcement_channel_id: String,
}

impl DiscordTrait for GuildConfig {}

#[allow(dead_code)]
impl GuildConfig {
    pub async fn marshal(&self, ctx: &BotContext<'_>) -> Result<Role, BotError> {
        Self::to_role(ctx, &self.marshal_role_id).await
    }

    pub async fn log_channel(&self, ctx: &BotContext<'_>) -> Result<GuildChannel, BotError> {
        Self::to_channel(ctx, &self.log_channel_id).await
    }

    pub async fn mail_channel(&self, ctx: &BotContext<'_>) -> Result<GuildChannel, BotError> {
        Self::to_channel(ctx, &self.mail_channel_id).await
    }
    pub async fn announcement_channel(
        &self,
        ctx: &BotContext<'_>,
    ) -> Result<GuildChannel, BotError> {
        Self::to_channel(ctx, &self.announcement_channel_id).await
    }

    pub async fn update_mail_channel(
        &self,
        ctx: &BotContext<'_>,
        channel: &Channel,
    ) -> Result<(), BotError> {
        let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;
        ctx.data()
            .database
            .update_mail_channel(&guild_id, &channel.id())
            .await?;
        Ok(())
    }
}

/// The status of a tournament. Used to know if a tournament should be paused, retired, etc.
#[derive(Debug, PartialEq, Eq, sqlx::Type, Serialize, Deserialize, Display, Default, Clone)]
#[sqlx(type_name = "tournament_status", rename_all = "snake_case")]
pub enum TournamentStatus {
    #[strum(to_string = "Open")]
    Pending,
    #[strum(to_string = "In progress")]
    Started,
    #[strum(to_string = "Paused")]
    Paused,
    #[strum(to_string = "Inactive ")]
    #[default]
    Inactive,
}

/// A tournament within the database.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Tournament {
    pub tournament_id: i32,
    pub name: String,
    pub guild_id: String,
    pub rounds: i32,
    pub current_round: i32,
    pub created_at: i64,
    pub start_time: Option<i64>,
    pub status: TournamentStatus,
    pub tournament_role_id: Option<String>,
    #[serde(default)]
    pub mode: Mode,
    pub map: BrawlMap,
    pub wins_required: i32,
    pub announcement_channel_id: String,
    pub notification_channel_id: String,
}

impl DiscordTrait for Tournament {}

#[allow(dead_code)]
impl Tournament {
    pub async fn announcement_channel(
        &self,
        ctx: &BotContext<'_>,
    ) -> Result<GuildChannel, BotError> {
        Ok(Self::to_channel(ctx, &self.announcement_channel_id)
            .await
            .map_err(|_| ChannelNotExists(self.announcement_channel_id.clone()))?)
    }
    pub async fn notification_channel(
        &self,
        ctx: &BotContext<'_>,
    ) -> Result<GuildChannel, BotError> {
        Ok(Self::to_channel(ctx, &self.notification_channel_id)
            .await
            .map_err(|_| ChannelNotExists(self.notification_channel_id.clone()))?)
    }

    pub async fn player_role(&self, ctx: &BotContext<'_>) -> Result<Option<Role>, BotError> {
        if let Some(role) = &self.tournament_role_id {
            return Ok(Some(
                Self::to_role(ctx, role)
                    .await
                    .map_err(|_| RoleNotExists(role.to_string()))?,
            ));
        }

        Ok(None)
    }

    #[inline]
    pub fn is_paused(&self) -> bool {
        self.status == TournamentStatus::Paused
    }

    #[inline]
    pub fn is_started(&self) -> bool {
        self.status == TournamentStatus::Started
    }

    #[inline]
    pub fn is_pending(&self) -> bool {
        self.status == TournamentStatus::Pending
    }

    #[inline]
    pub fn is_inactive(&self) -> bool {
        self.status == TournamentStatus::Inactive
    }

    pub async fn count_players_in_current_round(
        &self,
        ctx: &BotContext<'_>,
    ) -> Result<i64, BotError> {
        ctx.data()
            .database
            .participants(self.tournament_id, self.current_round)
            .await
    }

    pub async fn count_finished_matches(&self, ctx: &BotContext<'_>) -> Result<i64, BotError> {
        ctx.data()
            .database
            .count_finished_matches(self.tournament_id, self.current_round)
            .await
    }

    pub async fn set_announcement_channel(
        &self,
        ctx: &BotContext<'_>,
        channel: &Channel,
    ) -> Result<(), BotError> {
        ctx.data()
            .database
            .set_announcement_channel(self.tournament_id, &channel.id())
            .await
    }

    pub async fn set_notification_channel(
        &self,
        ctx: &BotContext<'_>,
        channel: &Channel,
    ) -> Result<(), BotError> {
        ctx.data()
            .database
            .set_notification_channel(self.tournament_id, &channel.id())
            .await
    }

    pub async fn set_player_role(&self, ctx: &BotContext<'_>, role: &Role) -> Result<(), BotError> {
        ctx.data()
            .database
            .set_player_role(self.tournament_id, &role.id)
            .await
    }

    pub async fn update(&mut self, ctx: &BotContext<'_>) -> Result<(), BotError> {
        let guild_id = ctx.guild_id().ok_or(NotInAGuild)?;
        let tournament = ctx
            .data()
            .database
            .get_tournament(&guild_id, self.tournament_id)
            .await?
            .ok_or(TournamentNotExists(self.tournament_id.to_string()))?;
        *self = tournament;
        Ok(())
    }

    pub async fn count_registers(self, ctx: &BotContext<'_>) -> Result<i64, BotError> {
        ctx.data()
            .database
            .count_registers(self.tournament_id)
            .await
    }
}

impl Selectable for Tournament {
    #[inline]
    fn label(&self) -> String {
        self.name.clone()
    }

    #[inline]
    fn identifier(&self) -> String {
        self.tournament_id.to_string()
    }
}

/// A Discord user within the database.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Player {
    pub discord_id: String,
    pub discord_name: String,
    pub player_tag: String,
    pub player_name: String,
    pub icon: i32,
    pub trophies: i32,
    pub brawlers: sqlx::types::JsonValue, // For match-level brawler bans. Not currently implemented
    pub deleted: bool,
}

impl DiscordTrait for Player {}
#[allow(dead_code)]
impl Player {
    #[inline]
    pub async fn user(&self, ctx: &BotContext<'_>) -> Result<User, BotError> {
        Self::to_user(ctx, &self.discord_id).await
    }

    #[inline]
    pub fn user_id(&self) -> Result<UserId, BotError> {
        Self::to_user_id(&self.discord_id)
    }

    #[inline]
    pub fn brawlers(&self) -> Vec<Brawler> {
        serde_json::from_value::<Vec<Brawler>>(self.brawlers.clone()).unwrap_or_default()
    }

    #[inline]
    pub fn icon(&self) -> String {
        format!(
            "https://cdn.brawlify.com/profile-icon/regular/{}.png",
            self.icon
        )
    }
}
/// A relational object that links a Discord user to a tournament they've joined.
#[derive(Serialize, Deserialize)]
pub struct TournamentPlayer {
    pub tournament_id: i32,
    pub discord_id: String,
}

impl DiscordTrait for TournamentPlayer {}

#[allow(dead_code)]
impl TournamentPlayer {
    #[inline]
    pub async fn user(&self, ctx: &BotContext<'_>) -> Result<User, BotError> {
        Self::to_user(ctx, &self.discord_id).await
    }

    #[inline]
    pub fn user_id(&self) -> Result<UserId, BotError> {
        Self::to_user_id(&self.discord_id)
    }
}

/// A match within the database, associated with a particular tournament.
///
/// Also known as a bracket to avoid conflicting with the Rust keyword.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Match {
    pub match_id: String,
    pub match_players: Vec<MatchPlayer>,
    pub score: String,
    pub winner: Option<String>,
    pub start: Option<i64>,
    pub end: Option<i64>,
}

impl DiscordTrait for Match {}

impl Match {
    pub fn new(
        tournament_id: i32,
        round: i32,
        sequence_in_round: i32,
        match_players: Vec<MatchPlayer>,
        score: &str,
    ) -> Self {
        Self {
            match_id: Self::generate_id(tournament_id, round, sequence_in_round),
            match_players,
            winner: None,
            score: score.to_string(),
            start: None,
            end: None,
        }
    }

    pub fn generate_id(tournament_id: i32, round: i32, sequence_in_round: i32) -> String {
        format!("{}.{}.{}", tournament_id, round, sequence_in_round)
    }

    /// Retrieves the winning player as a reference to its User type.
    /// The caller is responsible to clone or take ownership of the underlying User type.
    ///
    /// Warning: Cloning may be expensive as the user type contains image data in the form of bytes.
    pub fn get_winning_player(&self) -> Option<&MatchPlayer> {
        let winner_id = match self.winner {
            Some(ref id) => id,
            None => return None,
        };
        self.find_player(
            |p| p.discord_id == *winner_id,
            "Error: Unable to find winning player".to_string(),
        )
        .ok()
    }

    /// Retrieves the losing player as a reference to its User type.
    /// The caller is responsible to clone or take ownership of the underlying User type.
    ///
    /// Warning: Cloning may be expensive as the user type contains image data in the form of bytes.
    pub fn get_losing_player(&self) -> Option<&MatchPlayer> {
        let winner_id = match &self.winner {
            Some(id) => id,
            None => return None,
        };
        self.find_player(
            |p| p.discord_id != *winner_id,
            "Error: Unable to find losing player".to_string(),
        )
        .ok()
    }

    pub fn get_player(&self, discord_id: &str) -> Result<&MatchPlayer, BotError> {
        self.find_player(
            |p| p.discord_id == discord_id,
            format!(
                "Error: Unable to find player with Discord ID {} in match {}",
                discord_id, self.match_id
            ),
        )
    }

    pub fn get_opponent(&self, discord_id: &str) -> Result<&MatchPlayer, BotError> {
        self.find_player(
            |p| p.discord_id != discord_id,
            format!(
                "Error: Unable to find opponent for player with Discord ID {} in match {}",
                discord_id, self.match_id
            ),
        )
    }

    fn find_player<F>(&self, predicate: F, error_message: String) -> Result<&MatchPlayer, BotError>
    where
        F: Fn(&&MatchPlayer) -> bool,
    {
        self.match_players
            .iter()
            .find(predicate)
            .ok_or(UserNotExists(error_message).into())
    }

    pub fn tournament(&self) -> Result<i32, BotError> {
        Ok(self
            .match_id
            .split('.')
            .nth(0)
            .ok_or(TournamentNotExists(self.match_id.clone()))?
            .parse::<i32>()?)
    }

    pub fn round(&self) -> Result<i32, BotError> {
        Ok(self
            .match_id
            .split('.')
            .nth(1)
            .ok_or(RoundNotExists(self.match_id.clone()))?
            .parse::<i32>()?)
    }

    pub fn sequence(&self) -> Result<i32, BotError> {
        Ok(self
            .match_id
            .split('.')
            .nth(2)
            .ok_or(MatchNotExists(self.match_id.clone()))?
            .parse::<i32>()?)
    }

    #[inline]
    pub async fn winner(&self, ctx: &BotContext<'_>) -> Result<Option<User>, BotError> {
        match &self.winner {
            Some(winner) => Ok(Some(Self::to_user(ctx, winner).await?)),
            None => Ok(None),
        }
    }

    #[inline]
    pub fn is_not_bye(&self) -> bool {
        self.match_players.len() == 2
    }

    pub fn is_valid_id(id: &str) -> bool {
        let parts: Vec<&str> = id.split('.').collect();
        parts.len() == 3
            && parts[0].parse::<i32>().is_ok()
            && parts[1].parse::<i32>().is_ok()
            && parts[2].parse::<i32>().is_ok()
    }

    pub fn is_valid_score(score: &str) -> bool {
        let [left, right] = score.split('-').collect::<Vec<&str>>()[..] else {
            return false;
        };
        let left_num = left.parse::<i32>();
        let right_num = right.parse::<i32>();
        match (left_num, right_num) {
            (Ok(left), Ok(right)) => left >= 0 && right >= 0 && left > right,
            (Err(_), Err(_)) => true,
            _ => false,
        }
    }

    pub fn parts(id: &str) -> Result<(i32, i32, i32), BotError> {
        let parts: Vec<&str> = id.split('.').collect();
        if parts.len() != 3 {
            return Err(MatchNotExists(id.to_string()).into());
        }
        Ok((
            parts[0].parse::<i32>()?,
            parts[1].parse::<i32>()?,
            parts[2].parse::<i32>()?,
        ))
    }
}

/// A relational entity linking players to matches.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MatchPlayer {
    pub match_id: String,
    pub discord_id: String,
    pub player_type: PlayerType,
    pub ready: bool,
}

impl From<Player> for MatchPlayer {
    fn from(value: Player) -> Self {
        MatchPlayer {
            match_id: "".to_string(),
            discord_id: value.discord_id,
            player_type: PlayerType::Player,
            ready: false,
        }
    }
}

impl DiscordTrait for MatchPlayer {}

impl MatchPlayer {
    #[inline]
    pub async fn to_user(&self, ctx: &BotContext<'_>) -> Result<User, BotError> {
        Ok(self.user_id()?.to_user(ctx).await?)
    }
    #[inline]
    pub fn user_id(&self) -> Result<UserId, BotError> {
        Self::to_user_id(&self.discord_id)
    }
}

/// The type of player within a match.
///
/// Used to determine if the slot is occupied by a real player, a dummy (a bye round),
/// or is pending (waiting for a player to reach the bracket).
#[derive(Debug, sqlx::Type, Serialize, Deserialize, Clone)]
#[sqlx(type_name = "player_type", rename_all = "snake_case")]
pub enum PlayerType {
    Player,
    Dummy,
    Pending,
}

/// A match schedule within the database.
#[derive(Serialize, Deserialize)]
pub struct MatchSchedule {
    match_id: String,
    proposed_time: i32,
    time_of_proposal: chrono::DateTime<chrono::Utc>,
    proposer: Option<i32>,
    accepted: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BattleRecord {
    pub record_id: i64,
    #[serde(default)]
    pub match_id: String,
    #[serde(default)]
    pub battles: Vec<Battle>,
}

impl BattleRecord {
    pub fn new(
        ctx: &BotContext<'_>,
        match_id: String,
        battles: Vec<crate::api::official_brawl_stars::BattleLogItem>,
    ) -> Self {
        Self {
            record_id: ctx.now().timestamp(),
            match_id,
            battles: battles.into_iter().map(Battle::from).collect(),
        }
    }

    pub async fn execute(&self, ctx: &BotContext<'_>) -> Result<(), BotError> {
        let db = &ctx.data().database;
        let record = db.add_record(self).await?;
        for battle in &self.battles {
            let id = db.add_battle(battle, record).await?;
            db.add_event(&battle.event, id).await?;
            db.add_battle_class(&battle.battle_class, id).await?;
        }
        Ok(())
    }
}
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Battle {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub record_id: i64,

    pub battle_time: i64,
    pub battle_class: BattleClass,
    pub event: Event,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BattleClass {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub battle_id: i64,
    #[serde(default)]
    pub mode: Mode,
    pub battle_type: BattleType,
    pub result: BattleResult,
    pub duration: i32,
    pub trophy_change: Option<i32>,
    pub teams: serde_json::Value,
}

impl BattleClass {
    pub fn teams(&self) -> Vec<Vec<TeamPlayer>> {
        serde_json::from_value(self.teams.clone()).unwrap_or_default()
    }
}
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Event {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub mode: Mode,
    pub map: BrawlMap, // Optional because map can be NULL in the database
    #[serde(default)]
    pub battle_id: i64,
}

#[derive(Debug, Serialize, Deserialize, Eq, Clone)]
pub struct BrawlMap {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub disabled: bool,
}

impl Default for BrawlMap {
    fn default() -> Self {
        BrawlMap {
            id: 0,
            name: "Any".to_string(),
            disabled: false,
        }
    }
}

impl PartialEq for BrawlMap {
    fn eq(&self, other: &Self) -> bool {
        match self.id {
            0 => true,
            _ => self.id == other.id,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

#[allow(non_camel_case_types)]
#[derive(
    Debug,
    Default,
    sqlx::Type,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Display,
    poise::ChoiceParameter,
    EnumIter,
)]
#[sqlx(type_name = "mode", rename_all = "snake_case")]
pub enum Mode {
    #[name = "Brawl Ball"]
    #[strum(to_string = "Brawl Ball")]
    brawlBall,
    #[name = "Gem Grab"]
    #[strum(to_string = "Gem Grab")]
    gemGrab,
    #[name = "Heist"]
    #[strum(to_string = "Heist")]
    heist,
    #[name = "Bounty"]
    #[strum(to_string = "Bounty")]
    bounty,
    #[name = "Siege"]
    #[strum(to_string = "Siege")]
    siege,
    #[name = "Solo Showdown"]
    #[strum(to_string = "Solo Showdown")]
    soloShowdown,
    #[name = "Duo Showdown"]
    #[strum(to_string = "Duo Showdown")]
    duoShowdown,
    #[name = "Trio Showdown"]
    #[strum(to_string = "Trio Showdown")]
    trioShowdown,
    #[name = "Hot Zone"]
    #[strum(to_string = "Hot Zone")]
    hotZone,
    #[name = "Knockout"]
    #[strum(to_string = "Knockout")]
    knockout,
    #[name = "Takedown"]
    #[strum(to_string = "Takedown")]
    takedown,
    #[name = "Lone Star"]
    #[strum(to_string = "Lone Star")]
    loneStar,
    #[name = "Big Game"]
    #[strum(to_string = "Big Game")]
    bigGame,
    #[name = "Robo Rumble"]
    #[strum(to_string = "Robo Rumble")]
    roboRumble,
    #[name = "Boss Fight"]
    #[strum(to_string = "Boss Fight")]
    bossFight,
    #[name = "Wipeout"]
    #[strum(to_string = "Wipeout")]
    wipeout,
    #[name = "Duels"]
    #[strum(to_string = "Duels")]
    duels,
    #[name = "Paint Brawl"]
    #[strum(to_string = "Paint Brawl")]
    paintBrawl,
    #[name = "Brawl Ball 5v5"]
    #[strum(to_string = "Brawl Ball 5v5")]
    brawlBall5V5,
    #[name = "Gem Grab 5v5"]
    #[strum(to_string = "Gem Grab 5v5")]
    gemGrab5V5,
    #[name = "Wipeout 5v5"]
    #[strum(to_string = "Wipeout 5v5")]
    wipeout5V5,
    #[name = "Knockout 5v5"]
    #[strum(to_string = "Knockout 5v5")]
    knockOut5V5,
    #[name = "Unknown"]
    #[default]
    #[serde(other)]
    #[strum(to_string = "Unknown")]
    unknown,
}
impl Selectable for Mode {
    fn label(&self) -> String {
        self.to_string()
    }
    fn identifier(&self) -> String {
        match self {
            Mode::brawlBall => "brawlBall".to_string(),
            Mode::gemGrab => "gemGrab".to_string(),
            Mode::heist => "heist".to_string(),
            Mode::bounty => "bounty".to_string(),
            Mode::siege => "siege".to_string(),
            Mode::soloShowdown => "soloShowdown".to_string(),
            Mode::duoShowdown => "duoShowdown".to_string(),
            Mode::trioShowdown => "trioShowdown".to_string(),
            Mode::hotZone => "hotZone".to_string(),
            Mode::knockout => "knockout".to_string(),
            Mode::takedown => "takedown".to_string(),
            Mode::loneStar => "loneStar".to_string(),
            Mode::bigGame => "bigGame".to_string(),
            Mode::roboRumble => "roboRumble".to_string(),
            Mode::bossFight => "bossFight".to_string(),
            Mode::wipeout => "wipeout".to_string(),
            Mode::duels => "duels".to_string(),
            Mode::paintBrawl => "paintBrawl".to_string(),
            Mode::brawlBall5V5 => "brawlBall5V5".to_string(),
            Mode::gemGrab5V5 => "gemGrab5V5".to_string(),
            Mode::wipeout5V5 => "wipeout5V5".to_string(),
            Mode::knockOut5V5 => "knockOut5V5".to_string(),
            Mode::unknown => "unknown".to_string(),
        }
    }
}
impl Mode {
    pub fn from_string(mode: impl Into<String>) -> Self {
        match mode.into().as_str() {
            "bigGame" | "Big Game" => Self::bigGame,
            "bounty" | "Bounty" => Self::bounty,
            "bossFight" | "Boss Fight" => Self::bossFight,
            "brawlBall" | "Brawl Ball" => Self::brawlBall,
            "brawlBall5V5" | "Brawl Ball 5V5" => Self::brawlBall5V5,
            "duels" | "Duels" => Self::duels,
            "duoShowdown" | "Duo Showdown" => Self::duoShowdown,
            "gemGrab" | "Gem Grab" => Self::gemGrab,
            "gemGrab5V5" | "Gem Grab 5V5" => Self::gemGrab5V5,
            "heist" | "Heist" => Self::heist,
            "hotZone" | "Hot Zone" => Self::hotZone,
            "knockout" | "Knockout" => Self::knockout,
            "knockOut5V5" | "Knockout 5V5" => Self::knockOut5V5,
            "loneStar" | "Lone Star" => Self::loneStar,
            "paintBrawl" | "Paint Brawl" => Self::paintBrawl,
            "roboRumble" | "Robo Rumble" => Self::roboRumble,
            "siege" | "Siege" => Self::siege,
            "soloShowdown" | "Solo Showdown" => Self::soloShowdown,
            "takedown" | "Take Down" => Self::takedown,
            "wipeout" | "Wipeout" => Self::wipeout,
            "wipeout5V5" | "Wipeout 5V5" => Self::wipeout5V5,
            _ => Self::unknown,
        }
    }
    pub fn all() -> Vec<Mode> {
        Mode::iter().collect()
    }

    #[inline]
    pub fn is_unknown(&self) -> bool {
        *self == Mode::unknown
    }
}
#[allow(non_camel_case_types)]
#[derive(
    Debug, Default, sqlx::Type, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Display,
)]
#[sqlx(type_name = "battle_type", rename_all = "camelCase")]
pub enum BattleType {
    #[strum(to_string = "Ranked")]
    ranked,
    #[strum(to_string = "Friendly")]
    friendly,
    #[strum(to_string = "Unknown")]
    #[default]
    #[serde(other)]
    unknown,
}
#[allow(non_camel_case_types)]
#[derive(
    Debug, Default, sqlx::Type, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Display,
)]
#[sqlx(type_name = "result", rename_all = "camelCase")]
pub enum BattleResult {
    #[strum(to_string = "Victory")]
    victory,
    #[strum(to_string = "Defeat")]
    defeat,
    #[strum(to_string = "Draw")]
    draw,
    #[strum(to_string = "Unknown")]
    #[default]
    #[serde(other)]
    unknown,
}
