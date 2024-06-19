use std::{collections::HashMap, time::SystemTime};

use chrono::{DateTime, Utc};
use tokio_util::time::{delay_queue::Key, DelayQueue};

use crate::BotError;

#[derive(Debug)]
pub struct MatchRemindersQueue {
    pub timers: DelayQueue<String>,
    pub entries: HashMap<String, MatchReminderEntry>,
}

impl MatchRemindersQueue {
    pub fn new() -> Self {
        Self {
            timers: DelayQueue::new(),
            entries: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        match_id: &str,
        discord_id_1: &str,
        discord_id_2: &str,
        proposer: i16,
        notification_channel_id: &str,
        match_time: DateTime<Utc>,
    ) -> Result<(), BotError> {
        let delay = match_time - chrono::offset::Utc::now();
        let key = self.timers.insert(match_id.to_string(), delay.to_std()?);
        let match_reminder = MatchReminderEntry::new(
            match_id.to_string(),
            discord_id_1.to_string(),
            discord_id_2.to_string(),
            proposer,
            notification_channel_id.to_string(),
            match_time,
            key,
        );

        self.entries
            .insert(match_reminder.match_id.to_string(), match_reminder);

        Ok(())
    }

    pub fn update_accepted(&mut self, match_id: &str, accepted: bool) -> Result<(), BotError> {
        match self.entries.get_mut(match_id) {
            Some(entry) => entry.accepted = accepted,
            None => return Err(format!("Failed to update accepted status of match with ID {}: match reminder entry not found.", match_id).into()),
        };

        Ok(())
    }

    pub fn update_schedule(
        &mut self,
        match_id: &str,
        proposer: i16,
        match_time: DateTime<Utc>,
    ) -> Result<(), BotError> {
        let old_entry = match self.entries.remove(match_id) {
            Some(entry) => entry,
            None => {
                return Err(format!(
                    "Failed to update schedule of match with ID {}: match reminder entry not found",
                    match_id
                )
                .into())
            }
        };
        self.timers.remove(&old_entry.key);

        self.insert(
            match_id,
            &old_entry.discord_id_1,
            &old_entry.discord_id_2,
            proposer,
            &old_entry.notification_channel_id,
            match_time,
        )?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct MatchReminderEntry {
    pub match_id: String,
    pub discord_id_1: String,
    pub discord_id_2: String,
    pub proposer: i16,
    pub notification_channel_id: String,
    pub match_time: DateTime<Utc>,
    pub accepted: bool,
    pub key: Key,
}

impl MatchReminderEntry {
    pub fn new(
        match_id: String,
        discord_id_1: String,
        discord_id_2: String,
        proposer: i16,
        notification_channel_id: String,
        match_time: DateTime<Utc>,
        key: Key,
    ) -> Self {
        Self {
            match_id,
            discord_id_1,
            discord_id_2,
            proposer,
            notification_channel_id,
            match_time,
            accepted: false,
            key,
        }
    }
}
