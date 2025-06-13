#[cfg(test)]
mod tests {
    use anyhow::{bail, Context, Result};
    use serde_json::Value;
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::path::PathBuf;
    use tracing::info;
    use tracing_test::traced_test;
    use utilities::file_utils;
    use utilities::language::*;
    use wiktionary_en_entities::wiktionary_entity::*;

    fn parse_line(line: &String, i: usize) -> Result<DictionaryEntry> {
        parse_entry(line).with_context(|| format!("Couldn't parse line {} in DB file.", i))
    }

    #[test]
    fn word_is_always_present() -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        for (i, line) in file_reader.lines().enumerate().take(100) {
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
    #[traced_test]
    fn one_sound_always_set() -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        for (i, line) in file_reader.lines().enumerate().take(1000) {
            match line {
                Ok(ok_line) => {
                    let dictionary_entry = parse_line(&ok_line, i)?;
                    for sound in dictionary_entry.sounds {
                        if sound.other.is_some() && sound.enpr.is_some() {
                            info!("word: {}", dictionary_entry.word);
                            return Ok(());
                        }
                        //assert!(sound.other.is_some() || sound.enpr.is_some() || sound.ipa.is_some());
                    }
                }
                _ => bail!("couldn't read line {}", i),
            }
        }
        Ok(())
    }

    #[traced_test]
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
            info!("found key: {}", key);
        }
        Ok(())
    }

    fn lookup_array_item_fields_for(field: &str) -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut unique_keys = HashSet::new();
        for (i, line) in file_reader.lines().enumerate().take(100) {
            match line {
                Ok(ok_line) => {
                    let value: Value = serde_json::from_str(&ok_line)?;
                    if let Some(xs) = find_array_value_by(&value, field) {
                        for x in xs {
                            if let Some(obj) = x.as_object() {
                                for key in obj.keys() {
                                    unique_keys.insert(key.clone());
                                }
                            }
                        }
                    }
                }
                _ => bail!("couldn't read line {}", i),
            }
        }
        for key in unique_keys {
            info!("found key in {}: {}", field, key);
        }
        Ok(())
    }

    #[traced_test]
    #[test]
    fn lookup_synonym_fields() -> Result<()> {
        lookup_array_item_fields_for("synonyms")
    }

    #[traced_test]
    #[test]
    fn lookup_antonym_fields() -> Result<()> {
        lookup_array_item_fields_for("antonyms")
    }

    #[traced_test]
    #[test]
    fn lookup_sound_fields() -> Result<()> {
        lookup_array_item_fields_for("sounds")
    }

    #[traced_test]
    #[test]
    fn lookup_descendants_fields() -> Result<()> {
        lookup_array_item_fields_for("descendants")
    }

    fn explore_field_content_of_array_using_first_occurrence(
        array_field: &str,
        inner_field: &str,
    ) -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        for (i, line) in file_reader.lines().enumerate() {
            match line {
                Ok(ok_line) => {
                    let value: Value = serde_json::from_str(&ok_line)?;
                    let original_word = find_string_value_by_or_default(&value, "word");
                    if let Some(elem) = find_first_elem_in_array_by(&value, array_field) {
                        let word = find_string_value_by_or_default(&elem, "word");
                        if let Some(field_value) = find_string_value_by(&elem, inner_field) {
                            info!(
                                    "found word {} with field '{}' having an element '{}' with value '{}' reflecting '{}'",
                                    original_word, array_field, inner_field, field_value, word
                                );
                            return Ok(());
                        }
                    }
                }
                _ => bail!("couldn't read line {}", i),
            }
        }
        Ok(())
    }

    fn explore_definition_of_first_elem_in_an_array(field_name: &str) -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        for (i, line) in file_reader.lines().enumerate() {
            match line {
                Ok(ok_line) => {
                    let value: Value = serde_json::from_str(&ok_line)?;
                    let word = find_string_value_by_or_default(&value, "word");
                    if let Some(elem) = find_first_elem_in_array_by(&value, field_name) {
                        info!(
                            "for word '{}', found an elem in the array {}, defined as: '{}'",
                            word, field_name, elem
                        );
                        return Ok(());
                    }
                }
                _ => bail!("couldn't read line {}", i),
            }
        }
        Ok(())
    }

    #[traced_test]
    #[test]
    fn explore_sound_without_pronunciation_fields() -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        for (i, line) in file_reader.lines().enumerate() {
            match line {
                Ok(ok_line) => {
                    let value: Value = serde_json::from_str(&ok_line)?;
                    let word = find_string_value_by_or_default(&value, "word");
                    let sounds = find_array_value_by(&value, "sounds");
                    if let Some(sounds) = sounds {
                        for sound in sounds {
                            let ipa = find_string_value_by(&sound, "ipa");
                            let enpr = find_string_value_by(&sound, "enpr");
                            let other = find_string_value_by(&sound, "other");
                            if ipa.is_none() && enpr.is_none() && other.is_none() {
                                info!("for word '{}' found matching sound: {}", word, sound);
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

    #[traced_test]
    #[test]
    fn explore_first_descendant_found() -> Result<()> {
        explore_definition_of_first_elem_in_an_array("descendants")
    }

    #[traced_test]
    #[test]
    fn explore_field_content_of_sense_in_a_synonym_using_first_occurrence() -> Result<()> {
        explore_field_content_of_array_using_first_occurrence("synonyms", "sense")
    }

    #[traced_test]
    #[test]
    fn explore_field_content_of_sense_in_an_antonym_using_first_occurrence() -> Result<()> {
        explore_field_content_of_array_using_first_occurrence("antonyms", "sense")
    }

    #[traced_test]
    #[test]
    fn explore_field_content_of_note_in_a_sound_using_first_occurrence() -> Result<()> {
        explore_field_content_of_array_using_first_occurrence("sounds", "audio")
    }

    fn find_array_value_by(value: &Value, field: &str) -> Option<Vec<Value>> {
        if let Some(obj) = value.as_object() {
            for key in obj.keys() {
                if key == field {
                    return obj[key].as_array().cloned();
                }
            }
        }
        None
    }

    fn find_first_elem_in_array_by(value: &Value, field: &str) -> Option<Value> {
        if let Some(array) = find_array_value_by(value, field) {
            return array.get(0).cloned();
        }
        None
    }

    fn find_string_value_by(value: &Value, field: &str) -> Option<String> {
        if let Some(obj) = value.as_object() {
            for key in obj.keys() {
                if key == field {
                    return obj[key].as_str().map(|s| s.to_string());
                }
            }
        }
        None
    }

    fn find_string_value_by_or_default(value: &Value, field: &str) -> String {
        find_string_value_by(value, field).unwrap_or("<unknown>".to_string())
    }
}
