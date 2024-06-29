use std::fmt::Display;

use serde::{Deserialize, Serialize};

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
#[derive(Debug, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "tournament_status", rename_all = "snake_case")]
pub enum TournamentStatus {
    Pending,
    Started,
    Paused,
    Inactive,
}

impl Display for TournamentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TournamentStatus::Pending => write!(f, "Open for registration"),
            TournamentStatus::Started => write!(f, "In progress"),
            TournamentStatus::Paused => write!(f, "Paused"),
            TournamentStatus::Inactive => write!(f, "Inactive/Completed"),
        }
    }
}

/// A tournament within the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Tournament {
    pub tournament_id: i32,
    pub name: String,
    pub guild_id: String,
    pub rounds: i32,
    pub current_round: i32,
    pub created_at: i64,
    pub start_time: Option<i64>,
    pub status: TournamentStatus,
    pub map: Option<String>,
}

/// A Discord user within the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub discord_id: String,
    pub player_tag: String,
}

/// A relational object that links a Discord user to a tournamnet they've joined.
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
            player_1_type: PlayerType::Player,
            player_2_type: PlayerType::Player,
            discord_id_1,
            discord_id_2,
            player_1_ready: false,
            player_2_ready: false,
            winner: None,
        }
    }

    pub fn generate_id(tournament_id: &i32, round: &i32, sequence_in_round: &i32) -> String {
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
