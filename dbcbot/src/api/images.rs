use anyhow::anyhow;
use base64::{engine::general_purpose, Engine};
use cached::proc_macro::cached;
use reqwest::Client;
use serde_json::Value;
use tracing::debug;

use crate::{database, BotError};

use super::official_brawl_stars::Brawler;

#[derive(Debug)]
pub struct ImagesAPI {
    base_url: String,
    client: Client,
}

impl ImagesAPI {
    pub fn new() -> Self {
        Self {
            base_url: std::env::var("IMAGES_API")
                .expect("Expected IMAGES_API as an environment variable"),
            client: Client::new(),
        }
    }

    pub async fn match_image(
        self,
        player1: &database::models::User,
        player2: &database::models::User,
    ) -> Result<Vec<u8>, BotError> {
        let url = format!("{}/image/match", self.base_url);
        let payload = serde_json::json!({
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
        let bytes = get_image(url, payload).await?;
        Ok(bytes)
    }

    pub async fn result_image(
        self,
        winner: &database::models::User,
        loser: &database::models::User,
    ) -> Result<Vec<u8>, BotError> {
        let url = format!("{}/image/result", self.base_url);
        let payload = serde_json::json!({
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
        let bytes = get_image(url, payload).await?;
        Ok(bytes)
    }

    pub async fn profile_image(
        self,
        user: &database::models::User,
        tournament_id: String,
    ) -> Result<Vec<u8>, BotError> {
        let url = format!("{}/image/profile", self.base_url);
        let payload = serde_json::json!({
            "player": {
                "discord_id": user.discord_id,
                "discord_name": user.discord_name,
                "player_tag": user.player_tag,
                "player_name": user.player_name,
                "icon": user.icon,
                "trophies": user.trophies,
                "brawler_count": user.get_brawlers().len(),
                "tournament_id": tournament_id
            }
        });
        let bytes = get_image(url, payload).await?;
        Ok(bytes)
    }

    pub async fn battle_log(
        self,
        record: database::models::BattleRecord,
        matchid: database::models::Match,
    ) -> Result<Vec<u8>, BotError> {
        let url = format!("{}/images/battle_log", self.base_url);
        let data: Vec<Value> = record
            .battles
            .into_iter()
            .map(|battle| {
                let player1 = &battle.battle_class.teams[0][0];
                let player2 = &battle.battle_class.teams[1][0];
                serde_json::json!({})
            })
            .collect();
        let payloads = serde_json::json!({"battle_logs": data});

        let bytes = get_image(url, payloads).await?;
        Ok(bytes)
    }
}

#[cached(size = 50, time = 86400, result = true)]
async fn get_image(endpoint: String, payload: Value) -> Result<Vec<u8>, BotError> {
    let images_api = ImagesAPI::new();
    let response = images_api
        .client
        .get(endpoint)
        .header("accept", "text/plain")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;
    let content = match response.text().await {
        Ok(content) => {
            debug!("Successfully got image from API");
            content
        }
        Err(e) => {
            return Err(anyhow!(
                "Error getting image from API: {}\n{}",
                e,
                e.to_string()
            ));
        }
    };
    let bytes = match general_purpose::STANDARD.decode(content.clone()) {
        Ok(bytes) => bytes,
        Err(e) => {
            debug!("Error decoding image from API: {}\n{}", e, content);
            return Err(anyhow!(
                "Error decoding image from API: {}\n```json\n{}```",
                e,
                content
            ));
        }
    };
    Ok(bytes)
}
