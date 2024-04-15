use std::collections::HashMap;

use chrono::{DateTime, Utc};
use tokio_util::time::DelayQueue;
use uuid::Uuid;

use crate::BotError;

pub struct MatchReminders {
    pub reminder_times: DelayQueue<Uuid>,
    pub matches: HashMap<Uuid, MatchReminder>,
}

impl MatchReminders {
    pub fn new(reminder_times: DelayQueue<Uuid>, matches: HashMap<Uuid, MatchReminder>) -> Self {
        Self {
            reminder_times,
            matches,
        }
    }

    pub fn insert_reminder(&mut self, match_reminder: MatchReminder) -> Result<(), BotError> {
        let delay = match_reminder.match_time - chrono::offset::Utc::now();
        self.reminder_times
            .insert(match_reminder.match_id, delay.to_std()?);
        println!("Inserted match reminder: {:?}", match_reminder);
        self.matches.insert(match_reminder.match_id, match_reminder);

        Ok(())
    }
}

#[derive(Debug)]
pub struct MatchReminder {
    pub match_id: Uuid,
    pub discord_id_1: String,
    pub discord_id_2: String,
    pub guild_id: String,
    pub notification_channel_id: String,
    pub match_time: DateTime<Utc>,
}

// Might want to revisit this struct later
impl MatchReminder {
    pub fn new(
        match_id: Uuid,
        discord_id_1: String,
        discord_id_2: String,
        guild_id: String,
        notification_channel_id: String,
        match_time: DateTime<Utc>,
    ) -> Self {
        Self {
            match_id,
            discord_id_1,
            discord_id_2,
            guild_id,
            notification_channel_id,
            match_time,
        }
    }
}
