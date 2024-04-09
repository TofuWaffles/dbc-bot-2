use std::collections::HashMap;

use crate::BotError;
use poise::serenity_prelude::{self as serenity, futures::future::Map};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

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

pub enum ApiResult<M> {
    Ok(M),
    NotFound,
    Maintenance,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerProfile {
    tag: String,
    name: String,
    club: Club,
    icon: HashMap<String, u32>,
    trophies: u32,
    exp_level: u32,
    exp_points: u32,
    highest_trophies: u32,
    solo_victories: u32,
    duo_victories: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Club {
    tag: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BattleLog {
    items: Vec<BattleLogItem>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BattleLogItem {
    battle: Battle,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Battle {
    mode: String,
    result: String,
    duration: u32,
    teams: Vec<Vec<TeamPlayer>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TeamPlayer {
    tag: String,
    name: String,
}

/// The Brawl Stars API.
pub struct BrawlStarsApi {
    /// The API token used to authenticate with the Brawl Stars API. You can get your own from the [Brawl Stars API website](https://developer.brawlstars.com/).
    token: String,
    /// The reqwest client used to make HTTP requests to the Brawl Stars API.
    client: Client,
}

impl GameApi for BrawlStarsApi {
    type Error = BotError;

    fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
            client: Client::new(),
        }
    }

    async fn get_player(&self, player_tag: &str) -> Result<ApiResult<PlayerProfile>, Self::Error> {
        let endpoint = format!("https://bsproxy.royaleapi.dev/v1/players/%23{}", player_tag);

        let response = self
            .client
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(ApiResult::Ok(response.json().await?)),
            StatusCode::NOT_FOUND => Ok(ApiResult::NotFound),
            StatusCode::SERVICE_UNAVAILABLE => {
                Ok(ApiResult::Maintenance)
            }
            _ => Err(format!(
                "Failed to get player {} from API with status code {}",
                player_tag,
                response.status()
            )
            .into()),
        }
    }

    async fn get_battle_log(&self, player_tag: &str) -> Result<ApiResult<BattleLog>, Self::Error> {
        let endpoint = format!(
            "https://bsproxy.royaleapi.dev/v1/players/%23{}/battlelog",
            player_tag
        );

        let response = self
            .client
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(ApiResult::Ok(response.json().await.unwrap())),
            StatusCode::NOT_FOUND => Ok(ApiResult::NotFound),
            StatusCode::SERVICE_UNAVAILABLE => {
                Ok(ApiResult::Maintenance)
            }
            _ => Err(format!(
                "Failed to get battle log of player {} from API with status code {}",
                player_tag,
                response.status()
            )
            .into()),
        }
    }

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
            StatusCode::OK => {
                Ok(false)
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                Ok(true)
            }
            _ => Err(format!(
                "Failed to check maintenance with status code {}",
                response.status()
            )
            .into()),
        }
    }
}
