#[cfg(test)]
mod tests {
    use anyhow::Result;
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
            language: Language::EN,
        };
        let serialized = anyhow_serde::to_string(&history_entry)?;
        info!("serializes as: {}", serialized);
        return Ok(());
    }

    #[traced_test]
    #[test]
    fn test_deserialize_history_entry() -> Result<()> {
        let term = "Hello Word!";
        let language = Language::EN;
        let history_entry = HistoryEntry {
            term: term.to_string(),
            language,
        };
        let serialized = anyhow_serde::to_string(&history_entry)?;
        let deserialized: HistoryEntry = anyhow_serde::from_str(&serialized)?;
        assert_eq!(term, deserialized.term);
        assert_eq!(language, deserialized.language);

        return Ok(());
    }
}
