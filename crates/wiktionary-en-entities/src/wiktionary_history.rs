use chrono::serde::ts_seconds::deserialize as from_ts;
use chrono::serde::ts_seconds::serialize as to_ts;
use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};
use std::fmt;

#[macro_export]
macro_rules! history_collection {
    ($language:expr) => {
        format!("history_{}", $language)
    };
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryEntry {
    pub word: String,
    #[serde(serialize_with = "to_ts")]
    #[serde(deserialize_with = "from_ts")]
    pub last_seen_at: DateTime<Utc>,
    #[serde(serialize_with = "to_ts")]
    #[serde(deserialize_with = "from_ts")]
    pub now_seen_at: DateTime<Utc>,
    pub count: usize,
}

impl fmt::Display for HistoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "found {}", self.word)?;
        writeln!(f, "last seen at {}", self.last_seen_at)?;
        writeln!(f, "now seen at {}", self.now_seen_at)?;
        writeln!(f, "count is {}", self.count)?;
        Ok(())
    }
}

impl HistoryEntry {
    pub fn from(word: String) -> Self {
        let now = Utc::now();
        Self {
            word,
            now_seen_at: now,
            last_seen_at: now,
            count: 0,
        }
    }

    pub fn tick(&mut self) {
        self.last_seen_at = self.now_seen_at;
        self.now_seen_at = Utc::now();
        self.count += 1;
    }
}
