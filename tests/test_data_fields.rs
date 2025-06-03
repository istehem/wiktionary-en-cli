#[cfg(test)]
mod tests {
    use anyhow::{bail, Context, Result};
    use serde_json::Value;
    use std::collections::HashSet;
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
        for (i, line) in file_reader.lines().enumerate().take(10) {
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

    #[test]
    fn lookup_fields() -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut unique_keys = HashSet::new();
        for (i, line) in file_reader.lines().enumerate().take(100) {
            match line {
                Ok(ok_line) => {
                    let value: Value = serde_json::from_str(&ok_line)?;
                    if let Some(obj) = value.as_object() {
                        for key in obj.keys() {
                            unique_keys.insert(key.clone());
                        }
                    }
                }
                _ => bail!("couldn't read line {}", i),
            }
        }
        for key in unique_keys {
            println!("found key: {}", key);
        }
        Ok(())
    }

    #[test]
    fn lookup_synonym_fields() -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut unique_keys = HashSet::new();
        for (i, line) in file_reader.lines().enumerate().take(100) {
            match line {
                Ok(ok_line) => {
                    let value: Value = serde_json::from_str(&ok_line)?;
                    if let Some(obj) = value.as_object() {
                        for key in obj.keys() {
                            if key == "synonyms" {
                                if let Some(synonyms) = obj[key].as_array() {
                                    for synonym in synonyms {
                                        if let Some(obj) = synonym.as_object() {
                                            for key in obj.keys() {
                                                unique_keys.insert(key.clone());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => bail!("couldn't read line {}", i),
            }
        }
        for key in unique_keys {
            println!("found key in synonyms: {}", key);
        }
        Ok(())
    }

    #[test]
    fn explore_field_content_of_sense_in_a_synonym_using_first_occurrence() -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        for (i, line) in file_reader.lines().enumerate().take(100) {
            match line {
                Ok(ok_line) => {
                    let value: Value = serde_json::from_str(&ok_line)?;
                    if let Some(obj) = value.as_object() {
                        for key in obj.keys() {
                            let word = &obj["word"];
                            if key == "synonyms" {
                                if let Some(synonyms) = obj[key].as_array() {
                                    for synonym in synonyms {
                                        if let Some(obj) = synonym.as_object() {
                                            for key in obj.keys() {
                                                if key == "sense" {
                                                    println!("for word {}", word);
                                                    println!("synonym is {}", obj["word"]);
                                                    println!("field value {}", obj[key]);
                                                    return Ok(());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => bail!("couldn't read line {}", i),
            }
        }
        Ok(())
    }

    #[test]
    fn explore_field_content_of_synonyms_using_first_occurrence() -> Result<()> {
        let language = Language::SV;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        for (i, line) in file_reader.lines().enumerate() {
            match line {
                Ok(ok_line) => {
                    let value: Value = serde_json::from_str(&ok_line)?;
                    if let Some(obj) = value.as_object() {
                        for key in obj.keys() {
                            if key == "synonyms" {
                                println!("for word: {}", obj["word"]);
                                println!("field value is: {}", obj[key]);
                                return Ok(());
                            }
                        }
                    }
                }
                _ => bail!("couldn't read line {}", i),
            }
        }
        Ok(())
    }
}
