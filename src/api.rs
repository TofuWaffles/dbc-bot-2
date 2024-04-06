use crate::BotError;
use poise::serenity_prelude as serenity;
use reqwest::{Client, StatusCode};

#[allow(async_fn_in_trait)]
pub trait GameApi {
    type Error;
    type JsonValue;

    fn new(token: &str) -> Self;

    async fn get_player(&self, player_tag: &str) -> Result<Option<Self::JsonValue>, Self::Error>;

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
