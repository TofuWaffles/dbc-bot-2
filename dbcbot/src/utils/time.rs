use chrono::DateTime;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub struct BDateTime {
    pub datetime: i64,
}

impl BDateTime {
    // Method to convert from custom string format
    pub fn from_str(s: &str) -> Result<Self, chrono::ParseError> {
        let datetime = DateTime::parse_from_str(s, "%Y%m%dT%H%M%S%.3fZ")?;
        let unix = datetime.timestamp();
        Ok(BDateTime { datetime: unix})
    }

}
