use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerProfile {
    pub tag: String,
    pub name: String,
    pub club: Option<Club>,
    pub icon: HashMap<String, u32>,
    pub trophies: u32,
    #[serde(rename = "3vs3Victories")]
    pub three_vs_three_victories: u32,
    pub solo_victories: u32,
    pub duo_victories: u32,
    pub exp_level: u32,
    pub exp_points: u32,
    pub highest_trophies: u32,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BattleLogItem {
    pub battle: Battle,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Battle {
    pub mode: String,
    pub result: String,
    pub duration: u32,
    pub teams: Vec<Vec<TeamPlayer>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamPlayer {
    pub tag: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrawlerList {
    pub list: Vec<Brawler>,
    pub paging: (),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Brawler {
    id: i32,
    name: String,
    star_powers: Vec<StarPower>,
    gadgets: Vec<Gadget>,
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
