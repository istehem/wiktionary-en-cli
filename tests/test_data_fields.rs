#[cfg(test)]
mod tests {
    use anyhow::{bail, Context, Result};
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::path::PathBuf;
    use utilities::file_utils;
    use utilities::language::*;
    use wiktionary_en_entities::wiktionary_entity::*;

    fn parse_line(line: &String, i: usize) -> Result<DictionaryEntry> {
        parse_entry(line).with_context(|| format!("Couldn't parse line {} in DB file.", i))
    }

    #[test]
    fn word_is_always_present() -> Result<()> {
        let language = Language::SV;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        for (i, line) in file_reader.lines().enumerate() {
            match line {
                Ok(ok_line) => {
                    let dictionary_entry = parse_line(&ok_line, i)?;
                    assert_ne!(dictionary_entry.word, "");
                }
                _ => bail!("couldn't read line {}", i),
            }
        }
        Ok(())
    }
}
