#[cfg(test)]
mod tests {
    use anyhow::Result;
    use chrono::{Timelike, Utc};
    use tracing::info;
    use tracing_test::traced_test;
    use utilities::anyhow_serde;
    use utilities::language::Language;
    use wiktionary_en_entities::history_entry::HistoryEntry;

    #[traced_test]
    #[test]
    fn test_serialize_language() -> Result<()> {
        let language = Language::EN;
        info!("test for language: {}", language);
        let serialized = anyhow_serde::to_string(&language)?;
        info!("serializes as: {}", serialized);

        return Ok(());
    }

    #[traced_test]
    #[test]
    fn test_serialize_history_entry() -> Result<()> {
        let history_entry = HistoryEntry {
            word: "Hello Word!".to_string(),
            last_seen_at: Utc::now(),
            now_seen_at: Utc::now(),
            count: 0,
        };
        let serialized = anyhow_serde::to_string(&history_entry)?;
        info!("serializes as: {}", serialized);
        return Ok(());
    }

    #[traced_test]
    #[test]
    fn test_deserialize_history_entry() -> Result<()> {
        let term = "Hello Word!";
        let last_seen_at = Utc::now()
            .with_nanosecond(0)
            .expect("truncating nanoseconds to zero should always be valid");
        let history_entry = HistoryEntry {
            word: term.to_string(),
            now_seen_at: last_seen_at.clone(),
            last_seen_at,
            count: 0,
        };
        let serialized = anyhow_serde::to_string(&history_entry)?;
        let deserialized: HistoryEntry = anyhow_serde::from_str(&serialized)?;
        assert_eq!(term, deserialized.word);
        assert_eq!(last_seen_at, deserialized.last_seen_at);

        Ok(())
    }
}
