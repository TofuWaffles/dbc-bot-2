use chrono::DateTime;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct BattleDateTime {
    pub datetime: i64,
}

impl BattleDateTime {
    pub fn new(unix: i64) -> Self {
        BattleDateTime { datetime: unix }
    }
    // Method to convert from custom string format
    pub fn from_str(s: &str) -> Result<Self, chrono::ParseError> {
        let datetime = DateTime::parse_from_str(s, "%Y%m%dT%H%M%S%.3fZ")?;
        let unix = datetime.timestamp();
        Ok(BattleDateTime { datetime: unix })
    }

    pub fn to_rfc2822(&self) -> String {
        DateTime::from_timestamp(self.datetime, 0)
            .unwrap()
            .to_rfc2822()
    }
}
