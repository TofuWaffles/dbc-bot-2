use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::BotError;

use super::{APIResult, Endpoint};

#[derive(Debug)]
pub struct BrawlifyAPI {
    client: Client,
    endpoint: Endpoint,
}

impl BrawlifyAPI {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            endpoint: Endpoint::new("https://api.brawlify.com/v1/".to_string()),
        }
    }

    pub async fn get_maps(&self) -> Result<APIResult<()>, BotError> {
        let response = todo!();
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Map {
    pub id: i32,
    pub new: bool,
    pub disabled: bool,
    pub name: String,
    pub hash: String,
    pub version: i32,
    pub link: String,
    pub image_url: String,
    pub credit: Option<i32>,
    pub environment: Environment,
    pub game_mode: GameMode,
    pub last_active: Option<i32>,
    pub data_updated: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub id: i32,
    pub name: String,
    pub hash: String,
    pub version: i32,
    pub image_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameMode {
    pub id: i32,
    pub name: String,
    pub hash: String,
    pub version: i32,
    pub color: String,
    pub link: String,
    pub image_url: String,
}
