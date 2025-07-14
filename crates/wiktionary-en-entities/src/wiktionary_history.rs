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
    pub last_hit: DateTime<Utc>,
}

impl fmt::Display for HistoryEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "at {}", self.last_hit)?;
        writeln!(f, "{}", self.word)?;
        Ok(())
    }
}
