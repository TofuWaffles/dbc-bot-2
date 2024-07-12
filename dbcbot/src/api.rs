use crate::{
    database::{
        self,
        models::{BattleResult, Mode},
    },
    utils::time::BDateTime,
    BotError,
};
use anyhow::anyhow;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Describes the API that the bot will use to interact with the game.
///
/// While we are using this mainly for Brawl Stars, you can theoretically implement this trait for any game API.
#[allow(async_fn_in_trait)]
pub trait GameApi {
    /// The error type that the API can return. You can usually just use BotError.
    type Error;

    /// Creates a new instance of the API with the given token.
    fn new(token: &str) -> Self;

    /// Retrieves a player's profile along with all the player's information.
    async fn get_player(&self, player_tag: &str) -> Result<ApiResult<PlayerProfile>, Self::Error>;

    /// Retrieves a player's battle log.
    async fn get_battle_log(&self, player_tag: &str) -> Result<ApiResult<BattleLog>, Self::Error>;

    /// Checks if the game is under maintenance by making a request to the game's API.
    async fn check_maintenance(&self) -> Result<bool, Self::Error>;
}

/// Wrapper for the result of an API call.
pub enum ApiResult<M> {
    Ok(M),
    NotFound,
    Maintenance,
}

pub trait Convert<T> {
    fn convert(&self) -> T;
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon {
    pub id: i32,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerProfile {
    pub tag: String,
    pub name: String,
    pub club: Option<Club>,
    pub icon: Icon,
    pub trophies: u32,
    #[serde(rename = "3vs3Victories")]
    pub three_vs_three_victories: u32,
    pub solo_victories: u32,
    pub duo_victories: u32,
    pub exp_level: u32,
    pub exp_points: u32,
    pub highest_trophies: u32,
    pub brawlers: Vec<Brawler>,
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

impl Convert<database::models::BattleRecord> for BattleLog {
    fn convert(&self) -> database::models::BattleRecord {
        database::models::BattleRecord {
            record_id: 0,
            match_id: "".to_string(),
            battles: self.items.iter().map(|item| item.convert()).collect(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BattleLogItem {
    pub battle_time: String,
    pub event: database::models::Event,
    pub battle: Battle,
}

impl Convert<database::models::Battle> for BattleLogItem {
    fn convert(&self) -> database::models::Battle {
        database::models::Battle {
            id: 0,
            record_id: 0,
            battle_time: BDateTime::from_str(&self.battle_time).map_or_else(|_| 0, |f| f.datetime),
            battle_class: self.battle.convert(),
            event: self.event.clone(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Battle {
    pub mode: Mode,
    #[serde(rename = "type")]
    pub battle_type: String,
    #[serde(default)]
    pub rank: i64,
    #[serde(default)]
    pub result: BattleResult,
    pub trophy_change: Option<i64>,
    pub duration: Option<u32>,
    #[serde(default)]
    pub teams: Vec<Vec<TeamPlayer>>,
    #[serde(default)]
    pub players: Vec<TeamPlayer>,
}

impl Convert<database::models::BattleClass> for Battle {
    fn convert(&self) -> database::models::BattleClass {
        database::models::BattleClass {
            id: 0,
            battle_id: 0,
            trophy_change: self.trophy_change,
            mode: self.mode,
            battle_type: serde_json::from_str(&self.battle_type)
                .unwrap_or(database::models::BattleType::unknown),
            result: self.result,
            duration: self.duration.unwrap_or(0) as i64,
            teams: serde_json::to_value(&self.teams).unwrap_or(serde_json::Value::Null),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamPlayer {
    pub tag: String,
    pub name: String,
    pub brawler: Brawler,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brawler {
    pub id: i64,
    pub name: String,
}
/// The Brawl Stars API.
#[derive(Debug)]
pub struct BrawlStarsApi {
    /// The API token used to authenticate with the Brawl Stars API. You can get your own from the [Brawl Stars API website](https://developer.brawlstars.com/).
    token: String,
    /// The reqwest client used to make HTTP requests to the Brawl Stars API.
    client: Client,
}

impl GameApi for BrawlStarsApi {
    type Error = BotError;

    /// Create a new API client.
    fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
            client: Client::new(),
        }
    }

    /// Get a player's profile information from the API
    async fn get_player(&self, player_tag: &str) -> Result<ApiResult<PlayerProfile>, Self::Error> {
        let endpoint = |tag: &str| format!("https://bsproxy.royaleapi.dev/v1/players/%23{}", tag);
        let response = self
            .client
            .get(&endpoint(player_tag))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(ApiResult::Ok(response.json::<PlayerProfile>().await?)),
            StatusCode::NOT_FOUND => Ok(ApiResult::NotFound),
            StatusCode::SERVICE_UNAVAILABLE => Ok(ApiResult::Maintenance),
            _ => Err(anyhow!(
                "Failed to get player {} from API with status code {}",
                player_tag,
                response.status()
            )),
        }
    }

    /// Get the battle log of a particular player.
    async fn get_battle_log(&self, player_tag: &str) -> Result<ApiResult<BattleLog>, Self::Error> {
        let endpoint =
            |tag: &str| format!("https://bsproxy.royaleapi.dev/v1/players/%23{tag}/battlelog");
        let response = self
            .client
            .get(endpoint(player_tag))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(ApiResult::Ok({
                let a = response.json::<BattleLog>().await;
                info!("{:#?}", a);
                a?
            })),
            StatusCode::NOT_FOUND => Ok(ApiResult::NotFound),
            StatusCode::SERVICE_UNAVAILABLE => Ok(ApiResult::Maintenance),
            _ => Err(anyhow!(
                "Failed to get battle log of player {} from API with status code {}",
                player_tag,
                response.status()
            )),
        }
    }

    /// Check whether or not the game is currently undergoing maintenance.
    async fn check_maintenance(&self) -> Result<bool, Self::Error> {
        // Make some arbitrary request to the server; it doesn't matter what it is
        let endpoint = "https://bsproxy.royaleapi.dev/v1/events/rotation";

        let response = self
            .client
            .get(endpoint)
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
}
