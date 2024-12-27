use std::str::FromStr;

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    database::{
        self,
        models::{BattleResult, Mode, Selectable},
    },
    utils::time::BattleDateTime,
    BotError,
};
use anyhow::anyhow;

use super::{APIResult, Endpoint};

/// The official Brawl Stars API.
///
/// Used for most game-related resource queries. Some resources aren't provided by this API. Check
/// the Brawlify API to see if it's available there.
#[derive(Debug)]
pub struct BrawlStarsAPI {
    /// The API token used to authenticate with the Brawl Stars API. You can get your own from the [Brawl Stars API website](https://developer.brawlstars.com/).
    token: String,
    /// The reqwest client used to make HTTP requests to the Brawl Stars API.
    client: Client,
    /// The API endpoint to request resources from.
    endpoint: Endpoint,
}

#[allow(dead_code)]
impl BrawlStarsAPI {
    /// Create a new API client.
    pub fn new() -> Self {
        let token = std::env::var("BRAWL_STARS_TOKEN")
            .expect("Expected BRAWL_STARS_TOKEN as an environment variable");
        Self {
            token: token.to_string(),
            client: Client::new(),
            endpoint: Endpoint::new("https://bsproxy.royaleapi.dev/v1/".to_string()),
        }
    }

    /// Get a player's profile information from the API
    pub async fn get_player(&self, player_tag: &str) -> Result<APIResult<PlayerProfile>, BotError> {
        let response = self
            .client
            .clone()
            .get(
                self.endpoint
                    .append_path(&format!("players/%23{}", player_tag)),
            )
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        APIResult::from_response(response).await
    }

    /// Get the battle log of a particular player.
    pub async fn get_battle_log(&self, player_tag: &str) -> Result<APIResult<BattleLog>, BotError> {
        let response = self
            .client
            .clone()
            .get(
                self.endpoint
                    .append_path(&format!("players/%23{}/battlelog", player_tag)),
            )
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        APIResult::from_response(response).await
    }

    /// Check whether or not the game is currently undergoing maintenance.
    pub async fn check_maintenance(&self) -> Result<bool, BotError> {
        // Make some arbitrary request to the server; it doesn't matter what it is
        let response = self
            .client
            .clone()
            .get(self.endpoint.append_path("events/rotation"))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(false),
            StatusCode::SERVICE_UNAVAILABLE => Ok(true),
            _ => Err(anyhow!(
                "Failed to check maintenance with status code {}",
                response.status()
            )),
        }
    }

    pub async fn get_all_brawlers(&self) -> Result<APIResult<Vec<Brawler>>, BotError> {
        let response = self
            .client
            .clone()
            .get(self.endpoint.append_path("brawlers"))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        APIResult::from_response(response).await
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerProfile {
    #[serde(deserialize_with = "deserialize_tag")]
    pub tag: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_club")]
    pub club: Option<Club>,
    pub icon: Icon,
    pub trophies: i32,
    #[serde(rename = "3vs3Victories")]
    pub three_vs_three_victories: i32,
    pub solo_victories: i32,
    pub duo_victories: i32,
    pub exp_level: i32,
    pub exp_points: i32,
    pub highest_trophies: i32,
    pub brawlers: Vec<Brawler>,
}

impl PlayerProfile {
    pub fn icon(&self) -> String {
        format!(
            "https://cdn.brawlify.com/profile-icon/regular/{}.png",
            self.icon.id
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Icon {
    pub id: i32,
}

//official_brawl_stars
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapEvent {
    pub id: i32,
    pub map: String,
    #[serde(default)]
    pub mode: Mode,
}

impl MapEvent {
    pub fn into(self, battle_id: i64) -> database::models::Event {
        database::models::Event {
            id: 0,
            map: database::models::BrawlMap {
                id: self.id,
                name: self.map,
                disabled: false,
            },
            mode: self.mode,
            battle_id,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Club {
    pub tag: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BattleLog {
    pub items: Vec<BattleLogItem>,
}

impl From<BattleLog> for database::models::BattleRecord {
    fn from(value: BattleLog) -> Self {
        Self {
            record_id: 0,
            match_id: "".to_string(),
            battles: value
                .items
                .into_iter()
                .map(database::models::Battle::from)
                .collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BattleLogItem {
    pub battle_time: String,
    pub event: MapEvent,
    pub battle: Battle,
}

impl From<BattleLogItem> for database::models::Battle {
    fn from(value: BattleLogItem) -> Self {
        let id = BattleDateTime::from_str(&value.battle_time).map_or_else(|_| 0, |f| f.datetime);
        Self {
            id: 0,
            record_id: 0,
            battle_time: id,
            battle_class: value.battle.into(),
            event: value.event.into(id),
        }
    }
}

impl BattleLogItem {
    pub fn unix(&self) -> i64 {
        BattleDateTime::from_str(&self.battle_time).map_or_else(|_| 0, |f| f.datetime)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Battle {
    #[serde(default)]
    pub mode: Mode,
    #[serde(rename = "type")]
    pub battle_type: String,
    #[serde(default)]
    pub rank: i32,
    #[serde(default)]
    pub result: BattleResult,
    pub trophy_change: Option<i32>,
    pub duration: Option<i32>,
    #[serde(default)]
    pub teams: Vec<Vec<TeamPlayer>>,
    #[serde(default)]
    pub players: Vec<TeamPlayer>,
}

impl From<Battle> for database::models::BattleClass {
    fn from(value: Battle) -> Self {
        Self {
            id: 0,
            battle_id: 0,
            trophy_change: value.trophy_change,
            mode: value.mode,
            battle_type: serde_json::from_str(&value.battle_type)
                .unwrap_or(database::models::BattleType::unknown),
            result: value.result,
            duration: value.duration.unwrap_or(0),
            teams: serde_json::to_value(&value.teams).unwrap_or(serde_json::Value::Null),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TeamPlayer {
    #[serde(deserialize_with = "deserialize_tag")]
    pub tag: String,
    pub name: String,
    pub brawler: Brawler,
}

fn deserialize_tag<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let tag = String::deserialize(deserializer)?;

    // Strip the leading '#' if present
    if let Some(stripped) = tag.trim().strip_prefix('#') {
        Ok(stripped.to_string())
    } else {
        Ok(tag)
    }
}

fn deserialize_club<'de, D>(deserializer: D) -> Result<Option<Club>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match value {
        serde_json::Value::Object(map) if map.is_empty() => Ok(None), // Check for empty object
        _ => {
            // Deserialize into Option<Club>
            let club: Option<Club> =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            Ok(club)
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrawlerList {
    pub list: Vec<Brawler>,
    pub paging: (),
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Brawler {
    pub id: i32,
    pub name: String,
}

impl Selectable for Brawler {
    fn label(&self) -> String {
        self.name.clone()
    }

    fn identifier(&self) -> String {
        self.id.to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StarPower {
    id: i32,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gadget {
    id: i32,
    name: String,
}
