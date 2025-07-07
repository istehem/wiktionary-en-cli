use chrono::serde::ts_seconds::deserialize as from_ts;
use chrono::serde::ts_seconds::serialize as to_ts;
use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

#[macro_export]
macro_rules! history_collection {
    ($language:expr) => {
        format!("history_{}", $language)
    };
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryEntry {
    pub term: String,
    #[serde(serialize_with = "to_ts")]
    #[serde(deserialize_with = "from_ts")]
    pub last_hit: DateTime<Utc>,
}
