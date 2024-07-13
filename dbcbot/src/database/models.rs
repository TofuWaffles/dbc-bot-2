use serde::{Deserialize, Serialize};
use strum::Display;

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
    pub announcement_channel_id: String,
    pub notification_channel_id: String,
    pub log_channel_id: String,
}

/// The status of a tournament. Used to know if a tournament should be paused, retired, etc.
#[derive(Debug, PartialEq, Eq, sqlx::Type, Serialize, Deserialize, Display, Default)]
#[sqlx(type_name = "tournament_status", rename_all = "snake_case")]
pub enum TournamentStatus {
    #[strum(to_string = "Open for registration")]
    Pending,
    #[strum(to_string = "In progress")]
    Started,
    #[strum(to_string = "Paused")]
    Paused,
    #[strum(to_string = "Inactive/Completed")]
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
    pub map: Option<String>,
    pub wins_required: i32,
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
    pub brawlers: sqlx::types::JsonValue,
}

impl User {
    pub fn to_player(&self) -> Player {
        Player {
            discord_id: self.discord_id.clone(),
            player_tag: self.player_tag.clone(),
        }
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
        if discord_id == &self.discord_id_1.clone().unwrap_or_default() {
            return Some(PlayerNumber::Player1);
        } else if discord_id == &self.discord_id_2.clone().unwrap_or_default() {
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
    pub duration: i64,
    pub trophy_change: Option<i64>,
    pub teams: serde_json::Value, // Assuming teams is stored as JSONB
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct Event {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub mode: Mode,
    pub map: Option<String>, // Optional because map can be NULL in the database
    #[serde(default)]
    pub battle_id: i64,
}

#[allow(non_camel_case_types)]
#[derive(
    Debug, Default, sqlx::Type, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Display,
)]
#[sqlx(type_name = "mode", rename_all = "camelCase")]
pub enum Mode {
    #[strum(to_string = "Brawl Ball")]
    brawlBall,
    #[strum(to_string = "Gem Grab")]
    gemGrab,
    #[strum(to_string = "Heist")]
    heist,
    #[strum(to_string = "Bounty")]
    bounty,
    #[strum(to_string = "Siege")]
    siege,
    #[strum(to_string = "Solo Showdown")]
    soloShowdown,
    #[strum(to_string = "Duo Showdown")]
    duoShowdown,
    #[strum(to_string = "Hot Zone")]
    hotZone,
    #[strum(to_string = "Knockout")]
    knockout,
    #[strum(to_string = "Takedown")]
    takedown,
    #[strum(to_string = "Lone Star")]
    loneStar,
    #[strum(to_string = "Big Game")]
    bigGame,
    #[strum(to_string = "Robo Rumble")]
    roboRumble,
    #[strum(to_string = "Boss Fight")]
    bossFight,
    #[strum(to_string = "Wipeout")]
    wipeout,
    #[strum(to_string = "Duels")]
    duels,
    #[strum(to_string = "Paint Brawl")]
    paintBrawl,
    #[strum(to_string = "Brawl Ball 5v5")]
    brawlBall5V5,
    #[strum(to_string = "Gem Grab 5v5")]
    gemGrab5V5,
    #[strum(to_string = "Wipeout 5v5")]
    wipeout5V5,
    #[strum(to_string = "Knockout 5v5")]
    knockOut5V5,
    #[default]
    #[strum(to_string = "Unknown")]
    unknown,
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
