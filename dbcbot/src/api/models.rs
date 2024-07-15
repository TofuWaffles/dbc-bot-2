use serde::{Deserialize, Serialize};

use crate::{
    database::{
        self,
        models::{BattleResult, Mode},
    },
    utils::time::BDateTime,
};
pub trait Convert<T> {
    fn convert(&self) -> T;
}
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
            duration: self.duration.unwrap_or(0),
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
