#[cfg(test)]
mod tests {
    use anyhow::{bail, Context, Result};
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::path::PathBuf;
    use tracing::info;
    use tracing_test::traced_test;
    use utilities::file_utils;
    use utilities::language::*;
    use wiktionary_en_entities::dictionary_entry::DictionaryEntry;

    fn parse_line(line: &str, i: usize) -> Result<DictionaryEntry> {
        line.parse()
            .with_context(|| format!("Couldn't parse line {} in DB file.", i))
    }

    #[traced_test]
    #[test]
    fn at_least_as_many_unique_entries_as_all_entries() -> Result<()> {
        let language = Language::SV;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut unique_entries = HashSet::new();
        let mut count = 0;
        for (i, line) in file_reader.lines().enumerate() {
            match line {
                Ok(ok_line) => {
                    let dictionary_entry = parse_line(&ok_line, i)?;
                    unique_entries.insert(dictionary_entry.word);
                }
                _ => bail!("couldn't read line {}", i),
            }
            count += 1;
        }
        info!("there are {} total entries in the db file", count);
        info!(
            "there are {} unique entries in the db file",
            unique_entries.len()
        );
        assert!(count >= unique_entries.len());
        Ok(())
    }
}
