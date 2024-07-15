use crate::BotError;
use anyhow::anyhow;
use reqwest::{Client, Response, StatusCode};

pub mod models;

use models::{BattleLog, PlayerProfile};
use serde::de::DeserializeOwned;

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
