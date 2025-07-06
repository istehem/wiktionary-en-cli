use serde::{Deserialize, Serialize};

pub const HISTORY_COLLECTION: &str = "history";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryEntry {
    pub term: String,
    pub lang_code: String,
}
