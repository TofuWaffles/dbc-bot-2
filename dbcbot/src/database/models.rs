use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, IntoEnumIterator};

use crate::api::official_brawl_stars::Brawler;

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

/// The configuration for a guild within the database.
#[derive(Serialize, Deserialize)]
pub struct GuildConfig {
    pub guild_id: String,
    pub marshal_role_id: String,
    pub log_channel_id: String,
    pub announcement_channel_id: String,
}

/// The status of a tournament. Used to know if a tournament should be paused, retired, etc.
#[derive(Debug, PartialEq, Eq, sqlx::Type, Serialize, Deserialize, Display, Default)]
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
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Tournament {
    pub tournament_id: i32,
    pub name: String,
    pub guild_id: String,
    pub rounds: i32,
    pub current_round: i32,
    pub created_at: i64,
    pub start_time: Option<i64>,
    pub status: TournamentStatus,
    pub tournament_role_id: String,
    pub mode: Mode,
    pub map: BrawlMap,
    pub wins_required: i32,
    pub announcement_channel_id: String,
    pub notification_channel_id: String,
}

impl Selectable for Tournament {
    fn label(&self) -> String {
        self.name.clone()
    }
    fn identifier(&self) -> String {
        self.tournament_id.to_string()
    }
}

/// A Discord user within the database.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct User {
    pub discord_id: String,
    pub discord_name: String,
    pub player_tag: String,
    pub player_name: String,
    pub icon: i32,
    pub trophies: i32,
    pub brawlers: sqlx::types::JsonValue, // For match-level brawler bans. Not currently
                                          // implemented
}

impl From<User> for Player {
    fn from(value: User) -> Self {
        Self {
            discord_id: value.discord_id.clone(),
            player_tag: value.player_tag.clone(),
        }
    }
}

impl User {
    pub fn get_brawlers(&self) -> Vec<Brawler> {
        serde_json::from_value::<Vec<Brawler>>(self.brawlers.clone()).unwrap_or_default()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub discord_id: String,
    pub player_tag: String,
}
/// A relational object that links a Discord user to a tournament they've joined.
#[derive(Serialize, Deserialize)]
pub struct TournamentPlayer {
    pub tournament_id: i32,
    pub discord_id: String,
}

/// A match within the database, associated with a particular tournament.
///
/// Also known as a bracket to avoid conflicting with the Rust keyword.
#[derive(Debug, Serialize, Deserialize)]
pub struct Match {
    pub match_id: String,
    pub tournament_id: i32,
    pub round: i32,
    pub sequence_in_round: i32,
    pub player_1_type: PlayerType,
    pub player_2_type: PlayerType,
    pub discord_id_1: Option<String>,
    pub discord_id_2: Option<String>,
    pub player_1_ready: bool,
    pub player_2_ready: bool,
    pub winner: Option<PlayerNumber>,
}

impl Match {
    pub fn new(
        match_id: String,
        tournament_id: i32,
        round: i32,
        sequence_in_round: i32,
        discord_id_1: Option<String>,
        discord_id_2: Option<String>,
    ) -> Self {
        Self {
            match_id,
            tournament_id,
            round,
            sequence_in_round,
            player_1_type: match discord_id_1 {
                Some(_) => PlayerType::Player,
                None => PlayerType::Dummy,
            },
            player_2_type: match discord_id_2 {
                Some(_) => PlayerType::Player,
                None => PlayerType::Dummy,
            },
            discord_id_1,
            discord_id_2,
            player_1_ready: false,
            player_2_ready: false,
            winner: None,
        }
    }

    pub fn generate_id(tournament_id: i32, round: i32, sequence_in_round: i32) -> String {
        format!("{}.{}.{}", tournament_id, round, sequence_in_round)
    }

    pub fn get_player_number(&self, discord_id: &str) -> Option<PlayerNumber> {
        if discord_id == self.discord_id_1.clone().unwrap_or_default() {
            return Some(PlayerNumber::Player1);
        } else if discord_id == self.discord_id_2.clone().unwrap_or_default() {
            return Some(PlayerNumber::Player2);
        }
        None
    }
}

/// The type of player within a match.
///
/// Used to determine if the slot is occupied by a real player, a dummy (a bye round),
/// or is pending (waiting for a player to reach the bracket).
#[derive(Debug, sqlx::Type, Serialize, Deserialize, PartialEq, Eq)]
#[sqlx(type_name = "player_type", rename_all = "snake_case")]
pub enum PlayerType {
    Player,
    Dummy,
    Pending,
}

#[derive(Debug, sqlx::Type, Serialize, Deserialize, PartialEq, Eq)]
#[sqlx(type_name = "player_number", rename_all = "snake_case")]
pub enum PlayerNumber {
    Player1,
    Player2,
}

impl ToString for PlayerNumber {
    fn to_string(&self) -> String {
        match self {
            PlayerNumber::Player1 => "player_1".to_string(),
            PlayerNumber::Player2 => "player_2".to_string(),
        }
    }
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

#[derive(Serialize, Deserialize)]
pub struct BattleRecord {
    pub record_id: i64,
    #[serde(default)]
    pub match_id: String,
    #[serde(default)]
    pub battles: Vec<Battle>,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BattleClass {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub battle_id: i64,
    pub mode: Mode,
    pub battle_type: BattleType,
    pub result: BattleResult,
    pub duration: i32,
    pub trophy_change: Option<i32>,
    pub teams: serde_json::Value, // Assuming teams is stored as JSONB
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct BrawlMap {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub name: String,
}

impl Default for BrawlMap {
    fn default() -> Self {
        BrawlMap {
            id: 0,
            name: "Any".to_string(),
        }
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
#[sqlx(type_name = "mode", rename_all = "camelCase")]
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
            "brawlBall" | "Brawl Ball" => Self::brawlBall,
            "gemGrab" => Self::gemGrab,
            "heist" => Self::heist,
            "bounty" => Self::bounty,
            "siege" => Self::siege,
            "soloShowdown" => Self::soloShowdown,
            "duoShowdown" => Self::duoShowdown,
            "hotZone" => Self::hotZone,
            "knockout" => Self::knockout,
            "takedown" => Self::takedown,
            "loneStar" => Self::loneStar,
            "bigGame" => Self::bigGame,
            "roboRumble" => Self::roboRumble,
            "bossFight" => Self::bossFight,
            "wipeout" => Self::wipeout,
            "duels" => Self::duels,
            "paintBrawl" => Self::paintBrawl,
            "brawlBall5V5" => Self::brawlBall5V5,
            "gemGrab5V5" => Self::gemGrab5V5,
            "wipeout5V5" => Self::wipeout5V5,
            "knockOut5V5" => Self::knockOut5V5,
            _ => Self::unknown,
        }
    }
    pub fn all() -> Vec<Mode> {
        Mode::iter().collect()
    }
}
#[allow(non_camel_case_types)]
#[derive(
    Debug, Default, sqlx::Type, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Display,
)]
#[sqlx(type_name = "type", rename_all = "camelCase")]
pub enum BattleType {
    #[strum(to_string = "Ranked")]
    ranked,
    #[strum(to_string = "Friendly")]
    friendly,
    #[strum(to_string = "Unknown")]
    #[default]
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
    unknown,
}
