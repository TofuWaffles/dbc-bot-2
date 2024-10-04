use reqwest::Client;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter};

use crate::{
    database::{
        self,
        models::{Mode, Selectable},
    },
    BotError,
};

use super::{official_brawl_stars::Brawler, APIResult, Endpoint};

/// Used to query map data from Brawl Stars.
///
/// At the moment, the official Brawl Stars API does not provide map data, which is why Brawlify is
/// needed.
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

    pub async fn get_maps(&self) -> Result<APIResult<Wrapper<BrawlMap>>, BotError> {
        let response = self
            .client
            .get(self.endpoint.append_path("maps"))
            .send()
            .await?;

        APIResult::from_response(response).await
    }

    pub async fn get_modes(&self) -> Result<APIResult<Wrapper<FullGameMode>>, BotError> {
        let response = self
            .client
            .get(self.endpoint.append_path("gamemodes"))
            .send()
            .await?;

        APIResult::from_response(response).await
    }

    pub async fn get_brawlers(&self) -> Result<APIResult<Wrapper<FullBrawler>>, BotError> {
        let response = self
            .client
            .get(self.endpoint.append_path("brawlers"))
            .send()
            .await?;

        APIResult::from_response(response).await
    }

    pub async fn get_map(&self, id: i32) -> Result<APIResult<BrawlMap>, BotError> {
        let response = self
            .client
            .get(self.endpoint.append_path(&format!("maps/{}", id)))
            .send()
            .await?;

        APIResult::from_response(response).await
    }

    pub async fn get_brawler(&self, id: i32) -> Result<APIResult<FullBrawler>, BotError> {
        let response = self
            .client
            .get(self.endpoint.append_path(&format!("brawlers/{}", id)))
            .send()
            .await?;
        APIResult::from_response(response).await
    }

    pub async fn get_mode(&self, id: i32) -> Result<APIResult<GameMode>, BotError> {
        let response = self
            .client
            .get(self.endpoint.append_path(&format!("gamemodes/{}", id)))
            .send()
            .await?;

        APIResult::from_response(response).await
    }
}

pub trait BrawlifyItem {
    fn name(&self) -> String;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct Wrapper<T> {
    pub list: Vec<T>,
}

impl<T> Wrapper<T>
where
    T: BrawlifyItem + std::cmp::Ord,
{
    pub fn sort_by_alphabet(self) -> Vec<T> {
        let mut items = self.list.into_iter().collect::<Vec<T>>();
        items.sort();
        items
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrawlMap {
    pub id: i32,
    pub new: bool,
    pub disabled: bool,
    pub name: String,
    pub hash: String,
    pub version: i32,
    pub link: String,
    pub image_url: String,
    pub credit: Option<String>,
    pub environment: Environment,
    pub game_mode: GameMode,
    pub last_active: Option<i32>,
    pub data_updated: Option<i32>,
}

impl From<BrawlMap> for database::models::BrawlMap {
    fn from(value: BrawlMap) -> database::models::BrawlMap {
        database::models::BrawlMap {
            id: value.id,
            name: value.name,
        }
    }
}

impl BrawlifyItem for BrawlMap {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Default for BrawlMap {
    fn default() -> Self {
        Self {
            id: 0,
            new: false,
            disabled: false,
            name: "Any".to_string(),
            hash: "Any".to_string(),
            version: 0,
            link: "".to_string(),
            image_url: "".to_string(),
            credit: None,
            environment: Environment {
                id: 0,
                name: "".to_string(),
                hash: "".to_string(),
                version: 0,
                image_url: "".to_string(),
            },
            game_mode: GameMode {
                id: None,
                name: "Any".to_string(),
                hash: "Any".to_string(),
                version: 0,
                color: "Any".to_string(),
                link: "".to_string(),
                image_url: "".to_string(),
            },
            last_active: None,
            data_updated: None,
        }
    }
}

impl Wrapper<BrawlMap> {
    pub fn filter_map_by_mode(self, mode: &Mode) -> Vec<BrawlMap> {
        let maps: Vec<BrawlMap> = self
            .list
            .into_iter()
            .filter(|m| {
                m.game_mode.name.to_lowercase() == mode.to_string().to_lowercase() && !m.disabled
            })
            .collect();
        println!("Found {} maps for mode {}", maps.len(), mode);
        maps
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub id: i32,
    pub name: String,
    pub hash: String,
    pub version: i32,
    pub image_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameMode {
    pub id: Option<i32>,
    pub name: String,
    pub hash: String,
    pub version: i32,
    pub color: String,
    pub link: String,
    pub image_url: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]

pub struct FullGameMode {
    pub id: Option<i64>,
    pub sc_id: i64,
    pub name: String,
    pub hash: String,
    pub sc_hash: String,
    pub disabled: bool,
    pub color: String,
    pub bg_color: String,
    pub version: i64,
    pub title: String,
    pub tutorial: String,
    pub description: String,
    pub short_description: String,
    pub sort1: i64,
    pub sort2: i64,
    pub link: String,
    pub image_url: String,
    pub image_url2: String,
    pub last_active: Option<String>,
}

impl From<FullGameMode> for GameMode {
    fn from(value: FullGameMode) -> Self {
        Self {
            id: value.id.map(|id| id as i32),
            name: value.name,
            hash: value.hash,
            version: value.version as i32,
            color: value.color,
            link: value.link,
            image_url: value.image_url,
        }
    }
}

impl From<GameMode> for database::models::Mode {
    fn from(value: GameMode) -> Self {
        match value.name.as_str() {
            "Bounty" => database::models::Mode::bounty,
            "Brawl Ball" => database::models::Mode::brawlBall,
            "Gem Grab" => database::models::Mode::gemGrab,
            "Heist" => database::models::Mode::heist,
            "Hot Zone" => database::models::Mode::hotZone,
            "Siege" => database::models::Mode::siege,
            "Solo Showdown" => database::models::Mode::soloShowdown,
            "Duo Showdown" => database::models::Mode::duoShowdown,
            "Trio Showdown" => database::models::Mode::trioShowdown,
            "Takedown" => database::models::Mode::takedown,
            "Lone Star" => database::models::Mode::loneStar,
            "Big Game" => database::models::Mode::bigGame,
            "Robo Rumble" => database::models::Mode::roboRumble,
            "Boss Fight" => database::models::Mode::bossFight,
            _ => database::models::Mode::unknown,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct FullBrawler {
    pub id: i32,
    pub name: String,
    pub rarity: Rarity,
    pub image_url: String,
    pub description: String,
}
impl From<FullBrawler> for Brawler {
    fn from(value: FullBrawler) -> Self {
        Self {
            id: value.id,
            name: value.name,
        }
    }
}

impl BrawlifyItem for FullBrawler {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Selectable for FullBrawler {
    fn identifier(&self) -> String {
        format!("{}", self.id)
    }

    fn label(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Serialize, Clone, EnumIter, Display)]
#[serde(rename_all = "camelCase")]
pub enum RarityEnum {
    #[strum(serialize = "Common")]
    Common = 1,
    #[strum(serialize = "Rare")]
    Rare,
    #[strum(serialize = "Super Rare")]
    SuperRare,
    #[strum(serialize = "Epic")]
    Epic,
    #[strum(serialize = "Mythic")]
    Mythic,
    #[strum(serialize = "Legendary")]
    Legendary,
}

impl<'de> Deserialize<'de> for RarityEnum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id: u32 = Deserialize::deserialize(deserializer)?;
        match id {
            1 => Ok(RarityEnum::Common),
            2 => Ok(RarityEnum::Rare),
            3 => Ok(RarityEnum::SuperRare),
            4 => Ok(RarityEnum::Epic),
            5 => Ok(RarityEnum::Mythic),
            6 => Ok(RarityEnum::Legendary),
            _ => Err(serde::de::Error::custom("Unknown rarity id")),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "camelCase")]
pub struct Rarity {
    pub id: i32,
    pub name: String,
    pub color: String,
}
