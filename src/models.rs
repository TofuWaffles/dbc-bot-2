use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ManagerRole {
    pub guild_id: String,
    pub manager_role_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub guild_id: String,
    pub marshal_role_id: String,
    pub announcement_channel_id: String,
    pub notification_channel_id: String,
    pub log_channel_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Tournament {
    pub tourmanet_id: i32,
    pub name: String,
    pub guild_id: String,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub active: bool,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub discord_id: String,
    pub player_tag: String,
}

#[derive(Serialize, Deserialize)]
pub struct TournamentPlayer {
    pub tournament_id: i32,
    pub discord_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Match {
    match_id: uuid::Uuid,
    tournament_id: i32,
    round: i32,
    sequence_in_round: i32,
    discord_id_1: String,
    discord_id_2: String,
    winner: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct MatchSchedule {
    match_id: uuid::Uuid,
    proposed_time: i32,
    time_of_proposal: chrono::DateTime<chrono::Utc>,
    proposer: Option<i32>,
    accepted: bool,
}
