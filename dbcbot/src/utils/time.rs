use std::str::FromStr;

use anyhow::anyhow;
use chrono::DateTime;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

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
        DateTime::from_timestamp(self.datetime, 0)
            .unwrap()
            .to_rfc2822()
    }
}

impl FromStr for BattleDateTime {
    type Err = BotError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (year, month, date, hour, minute, second) = (
            &s[0..4],
            &s[4..6],
            &s[6..8],
            &s[9..11],
            &s[11..13],
            &s[13..15],
        );
        let year: i32 = year.parse()?;
        let month: u32 = month.parse()?;
        let date: u32 = date.parse()?;
        let hour: u32 = hour.parse()?;
        let minute: u32 = minute.parse()?;
        let second: u32 = second.parse()?;
        let datetime = NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(year, month, date)
                .ok_or(anyhow!("Invalid format for date"))?,
            chrono::NaiveTime::from_hms_opt(hour, minute, second)
                .ok_or(anyhow!("Invalid format for time"))?,
        );
        Ok(Self {
            datetime: datetime.and_utc().timestamp(),
        })
    }
}
