use serde::{Deserialize, Serialize};
use utilities::language::*;

pub const HISTORY_COLLECTION: &str = "history";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryEntry {
    pub term: String,
    pub language: Language,
}
