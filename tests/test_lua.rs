#[cfg(test)]
mod tests {
    use anyhow::{bail, Context, Result};
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::path::PathBuf;
    use tracing::info;
    use tracing_test::traced_test;
    use utilities::file_utils;
    use utilities::language::*;
    use wiktionary_en_entities::wiktionary_entity::*;

    use wiktionary_en_lua;

    fn parse_line(line: &String) -> Result<DictionaryEntry> {
        parse_entry(line).with_context(|| format!("{}", "Couldn't parse line in DB file."))
    }

    #[traced_test]
    #[test]
    fn test_function() -> Result<()> {
        let result = wiktionary_en_lua::do_one_plus_one()?;
        assert!(result == 2);
        return Ok(());
    }

    #[traced_test]
    #[test]
    fn test_load_config() -> Result<()> {
        let config = wiktionary_en_lua::do_load_config()?;
        info!("lua returns a config with message: {}", &config.message);
        assert!(config.message == "Hello World!");
        assert!(config.language == Language::SV);
        return Ok(());
    }

    #[traced_test]
    #[test]
    fn test_intercept() -> Result<()> {
        let language = Language::FR;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let mut file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut line = String::new();
        match file_reader.read_line(&mut line) {
            Ok(_) => {
                let dictionary_entry = parse_line(&line)?;
                return wiktionary_en_lua::Config::intercept(&dictionary_entry);
            }
            _ => bail!("couldn't read line"),
        }
    }

    #[traced_test]
    #[test]
    fn test_intercept_several() -> Result<()> {
        let language = Language::FR;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut results = Vec::new();

        for (_index, line) in file_reader.lines().enumerate().take(10) {
            let dictionary_entry = parse_line(&line?)?;
            results.push(dictionary_entry);
        }

        return wiktionary_en_lua::intercept_witkionary_result(&results);
    }
}
