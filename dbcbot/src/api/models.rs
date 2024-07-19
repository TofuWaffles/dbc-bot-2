<<<<<<< HEAD
use serde::{Deserialize, Serialize};

use crate::{
    database::{
        self,
        models::{BattleResult, Mode},
    },
    utils::time::BDateTime,
};
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerProfile {
    pub tag: String,
    pub name: String,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Icon {
    pub id: i32,
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
            battles: value.items.into_iter().map(|item| database::models::Battle::from(item)).collect(),
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

impl From<BattleLogItem> for database::models::Battle {
    fn from(value: BattleLogItem) -> Self {
        Self {
            id: 0,
            record_id: 0,
            battle_time: BDateTime::from_str(&value.battle_time).map_or_else(|_| 0, |f| f.datetime),
            battle_class: value.battle.into(),
            event: value.event.clone(),
        }
    }
}

impl BattleLogItem{
    pub fn unix(&self) -> i64{
        BDateTime::from_str(&self.battle_time).map_or_else(|_| 0, |f| f.datetime)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Battle {
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamPlayer {
    pub tag: String,
    pub name: String,
    pub brawler: Brawler,
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
    id: i32,
    name: String,
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


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullBrawler{
    pub id: i32,
    pub name: String,
    pub rarity: Rarity,
    pub image_url: String,
    pub description: String,
}
impl From<FullBrawler> for Brawler{
    fn from(value: FullBrawler) -> Self {
        Self{
            id: value.id,
            name: value.name
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rarity{
    pub id: i32,
    pub name: String,
    pub color: String,
}
=======
use serde::{Deserialize, Serialize};

use crate::{
    database::{
        self,
        models::{BattleResult, Mode},
    },
    utils::time::BDateTime,
};
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerProfile {
    pub tag: String,
    pub name: String,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Icon {
    pub id: i32,
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
            battles: value.items.into_iter().map(|item| database::models::Battle::from(item)).collect(),
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

impl From<BattleLogItem> for database::models::Battle {
    fn from(value: BattleLogItem) -> Self {
        Self {
            id: 0,
            record_id: 0,
            battle_time: BDateTime::from_str(&value.battle_time).map_or_else(|_| 0, |f| f.datetime),
            battle_class: value.battle.into(),
            event: value.event.clone(),
        }
    }
}

impl BattleLogItem{
    pub fn unix(&self) -> i64{
        BDateTime::from_str(&self.battle_time).map_or_else(|_| 0, |f| f.datetime)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Battle {
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamPlayer {
    pub tag: String,
    pub name: String,
    pub brawler: Brawler,
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
    id: i32,
    name: String,
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

>>>>>>> bdb70236c68496a534164a78cf20d5436eba400c
