#[cfg(test)]
mod tests {
    use anyhow::Result;
    use chrono::{Timelike, Utc};
    use tracing::info;
    use tracing_test::traced_test;
    use utilities::anyhow_serde;
    use utilities::language::*;
    use wiktionary_en_entities::wiktionary_history::HistoryEntry;

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
            term: "Hello Word!".to_string(),
            last_hit: Utc::now(),
        };
        let serialized = anyhow_serde::to_string(&history_entry)?;
        info!("serializes as: {}", serialized);
        return Ok(());
    }

    #[traced_test]
    #[test]
    fn test_deserialize_history_entry() -> Result<()> {
        let term = "Hello Word!";
        let last_hit = Utc::now()
            .with_nanosecond(0)
            .expect("truncating nanoseconds to zero should always be valid");
        let history_entry = HistoryEntry {
            term: term.to_string(),
            last_hit,
        };
        let serialized = anyhow_serde::to_string(&history_entry)?;
        let deserialized: HistoryEntry = anyhow_serde::from_str(&serialized)?;
        assert_eq!(term, deserialized.term);
        assert_eq!(last_hit, deserialized.last_hit);

        return Ok(());
    }
}
