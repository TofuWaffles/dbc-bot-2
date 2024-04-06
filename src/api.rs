use crate::BotError;
use poise::serenity_prelude as serenity;
use reqwest::{Client, StatusCode};

/// Describes the API that the bot will use to interact with the game.
///
/// While we are using this mainly for Brawl Stars, you can theoretically implement this trait for any game API.
#[allow(async_fn_in_trait)]
pub trait GameApi {
    type Error;
    type JsonValue;

    /// Creates a new instance of the API with the given token.
    fn new(token: &str) -> Self;

    /// Retrieves a player's profile along with all the player's information.
    async fn get_player(&self, player_tag: &str) -> Result<Option<Self::JsonValue>, Self::Error>;

    /// Retrieves a player's battle log.
    async fn get_battle_log(
        &self,
        player_tag: &str,
    ) -> Result<Option<Self::JsonValue>, Self::Error>;
}

pub struct BrawlStarsApi {
    token: String,
    client: Client,
}

impl GameApi for BrawlStarsApi {
    type Error = BotError;
    type JsonValue = serenity::json::Value;

    fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
            client: Client::new(),
        }
    }

    async fn get_player(&self, player_tag: &str) -> Result<Option<Self::JsonValue>, Self::Error> {
        let endpoint = format!("https://bsproxy.royaleapi.dev/v1/players/%23{}", player_tag);

        let response = self
            .client
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(Some(response.json().await?)),
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(format!(
                "Failed to get player {} from API with status code {}",
                player_tag,
                response.status()
            )
            .into()),
        }
    }

    async fn get_battle_log(
        &self,
        player_tag: &str,
    ) -> Result<Option<Self::JsonValue>, Self::Error> {
        let endpoint = format!("https://bsproxy.royaleapi.dev/v1/players/%23{}/battlelog", player_tag);

        let response = self
            .client
            .get(&endpoint)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(Some(response.json().await?)),
            StatusCode::NOT_FOUND => Ok(None),
            _ => Err(format!(
                "Failed to get player {} from API with status code {}",
                player_tag,
                response.status()
            )
            .into()),
        }
    }
}
