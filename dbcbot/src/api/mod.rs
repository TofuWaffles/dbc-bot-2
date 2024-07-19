use crate::{database, BotError};
use anyhow::anyhow;
use base64::{
    engine::general_purpose,
    Engine,
};
use reqwest::{Client, Response, StatusCode};

pub mod models;

use models::{BattleLog, PlayerProfile};
use serde::{de::DeserializeOwned, Serialize};
use tracing::debug;
use tracing_subscriber::field::debug;
use self::models::Brawler;

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

    async fn get_all_brawlers(&self) -> Result<ApiResult<Vec<Brawler>>, Self::Error>;

    /// Checks if the game is under maintenance by making a request to the game's API.
    async fn check_maintenance(&self) -> Result<bool, Self::Error>;
}

/// Wrapper for the result of an API call.
pub enum ApiResult<M> {
    Ok(M),
    NotFound,
    Maintenance,
}

impl<M> ApiResult<M>
where
    M: DeserializeOwned,
{
    /// Create an API result from a response.
    ///
    /// If the response code is 200, an Ok variant will be returned containing the json data, which
    /// can then be deserialized into any type that implements Serialize.
    ///
    /// Errors if the response code is something that is either not covered by the API
    /// documentation or is not something that can be appropriately dealt with by the bot.
    pub async fn from_response(response: Response) -> Result<Self, BotError> {
        match response.status() {
            StatusCode::OK => Ok(ApiResult::Ok(response.json().await?)),
            StatusCode::NOT_FOUND => Ok(ApiResult::NotFound),
            StatusCode::SERVICE_UNAVAILABLE => Ok(ApiResult::Maintenance),
            _ => Err(anyhow!(
                "Request failed with status code: {}\n\nResponse details: {:#?}",
                response.status(),
                response
            )),
        }
    }
}

/// The API endpoint to retrieve resources from.
#[derive(Debug)]
pub struct Endpoint {
    url: String,
}

impl Endpoint {
    fn new(url: String) -> Self {
        Self { url }
    }
    /// Append a path to retrieve a specific resource from the endpoint. e.g. pass in
    /// format!("players/%23{}", player_tag) to get a specific player profile.
    ///
    /// Refer to the API documentation for the exact path.
    fn append_path(&self, path: &str) -> String {
        let mut full_url = self.url.clone();

        full_url.push_str(path);

        full_url
    }
}

/// The Brawl Stars API.
#[derive(Debug)]
pub struct BrawlStarsApi {
    /// The API token used to authenticate with the Brawl Stars API. You can get your own from the [Brawl Stars API website](https://developer.brawlstars.com/).
    token: String,
    /// The reqwest client used to make HTTP requests to the Brawl Stars API.
    client: Client,
    /// The API endpoint to request resources from.
    endpoint: Endpoint,
}

impl GameApi for BrawlStarsApi {
    type Error = BotError;

    /// Create a new API client.
    fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
            client: Client::new(),
            endpoint: Endpoint::new("https://bsproxy.royaleapi.dev/v1/".to_string()),
        }
    }

    /// Get a player's profile information from the API
    async fn get_player(&self, player_tag: &str) -> Result<ApiResult<PlayerProfile>, Self::Error> {
        let response = self
            .client
            .get(
                &self
                    .endpoint
                    .append_path(&format!("players/%23{}", player_tag)),
            )
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        ApiResult::from_response(response).await
    }

    /// Get the battle log of a particular player.
    async fn get_battle_log(&self, player_tag: &str) -> Result<ApiResult<BattleLog>, Self::Error> {
        let response = self
            .client
            .get(
                &self
                    .endpoint
                    .append_path(&format!("players/%23{}/battlelog", player_tag)),
            )
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        ApiResult::from_response(response).await
    }

    /// Check whether or not the game is currently undergoing maintenance.
    async fn check_maintenance(&self) -> Result<bool, Self::Error> {
        // Make some arbitrary request to the server; it doesn't matter what it is
        let response = self
            .client
            .get(&self.endpoint.append_path("events/rotation"))
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

    async fn get_all_brawlers(&self) -> Result<ApiResult<Vec<Brawler>>, Self::Error> {
        let response = self
            .client
            .get(&self.endpoint.append_path("brawlers"))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        ApiResult::from_response(response).await
    }
}

pub struct ImagesAPI {
    base_url: String,
    client: Client,
}

impl ImagesAPI {
    pub fn new() -> Result<Self, BotError> {
        Ok(Self {
            base_url: std::env::var("IMAGES_API")?,
            client: Client::new(),
        })
    }

    async fn get<T>(&self, endpoint: impl reqwest::IntoUrl, payload: &T) -> Result<Vec<u8>, BotError> where T: Serialize + ?Sized{
        let response = self
            .client
            .get(endpoint)
            .header("accept", "text/plain")
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await?;
        let content = match response.text().await {
            Ok(content) => {
                debug!("Successfully got image from API");
                content
            },
            Err(e) => {
                return Err(anyhow!("Error getting image from API: {}\n{}", e, e.to_string()));
            }
        };
        let bytes = match general_purpose::STANDARD.decode(content.clone()){
            Ok(bytes) => bytes,
            Err(e) => {
                debug!("Error decoding image from API: {}\n{}", e, content);
                return Err(anyhow!("Error decoding image from API: {}\n```json\n{}```", e, content));
            }
        };
        Ok(bytes)
    }

    pub async fn match_image(
        self,
        player1: &database::models::User,
        player2: &database::models::User,
    ) -> Result<Vec<u8>, BotError> {
        let url = format!("{}/image/match", self.base_url);
        let payload = &serde_json::json!({
            "player1": {
                "discord_id": player1.discord_id,
                "discord_name": player1.discord_name,
                "player_tag": player1.player_tag,
                "player_name": player1.player_name,
                "icon": player1.icon
            },
            "player2": {
                "discord_id": player2.discord_id,
                "discord_name": player2.discord_name,
                "player_tag": player2.player_tag,
                "player_name": player2.player_name,
                "icon": player2.icon
            }
        });
        let bytes = self.get(url, payload).await?;
        Ok(bytes)
    }

    pub async fn result_image(
        self,
        winner: &database::models::User,
        loser: &database::models::User,
    ) -> Result<Vec<u8>, BotError> {
        let url = format!("{}/image/result", self.base_url);
        let payload = &serde_json::json!({
            "winner": {
                "discord_id": winner.discord_id,
                "discord_name": winner.discord_name,
                "player_tag": winner.player_tag,
                "player_name": winner.player_name,
                "icon": winner.icon
            },
            "loser": {
                "discord_id": loser.discord_id,
                "discord_name": loser.discord_name,
                "player_tag": loser.player_tag,
                "player_name": loser.player_name,
                "icon": loser.icon
            }
        });
        let bytes = self.get(url, payload).await?;
        Ok(bytes)
    }

}
