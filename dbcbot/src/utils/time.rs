<<<<<<< HEAD
use std::str::FromStr;

use anyhow::anyhow;
use chrono::DateTime;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

use crate::BotError;
#[derive(Serialize, Deserialize, Debug)]
pub struct BattleDateTime {
    pub datetime: i64,
}

impl BattleDateTime {
    #[inline]
    pub fn new(unix: i64) -> Self {
        BattleDateTime { datetime: unix }
    }

    #[inline]
    pub fn to_rfc2822(&self) -> String {
       DateTime::from_timestamp(self.datetime, 0).unwrap().to_rfc2822()
    }
}


impl FromStr for BattleDateTime {
    type Err = BotError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (year, month, date, hour, minute, second) = (
            &s[0..4], &s[4..6], &s[6..8], &s[9..11], &s[11..13], &s[13..15],
        );
        let year: i32 = year.parse()?;
        let month: u32 = month.parse()?;
        let date: u32 = date.parse()?;
        let hour: u32 = hour.parse()?;
        let minute: u32 = minute.parse()?;
        let second: u32 = second.parse()?;
        let datetime = NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(year, month, date).ok_or(anyhow!("Invalid format for date"))?,
            chrono::NaiveTime::from_hms_opt(hour, minute, second).ok_or(anyhow!("Invalid format for time"))?,
        );
        Ok(Self{datetime: datetime.and_utc().timestamp()})
    }
}
=======
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
>>>>>>> 00eda06a4ff38f1d74b9574b1a12c5d41d3568fb
