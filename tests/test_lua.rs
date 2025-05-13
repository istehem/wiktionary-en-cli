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
    fn test_load_config() -> Result<()> {
        let config_handler = wiktionary_en_lua::ConfigHandler::init()?;
        let config = config_handler.config;
        info!("lua returns a config with message: {}", &config.message);
        assert!(config.message == "Hello World!");
        assert!(config.language == Some(Language::EN));
        return Ok(());
    }

    #[traced_test]
    #[test]
    fn test_intercept() -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let mut file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut line = String::new();
        match file_reader.read_line(&mut line) {
            Ok(_) => {
                //let dictionary_entry = parse_line(&line)?;
                //let _ = wiktionary_en_lua::Config::intercept(&dictionary_entry)?;
                // let returned_dictionary_entry =
                //    wiktionary_en_lua::Config::intercept(&dictionary_entry)?;
                //println!("{}", returned_dictionary_entry.to_pretty_string());
                return Ok(());
            }
            _ => bail!("couldn't read line"),
        }
    }

    #[traced_test]
    #[test]
    fn test_intercept_several() -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut results = Vec::new();

        for (_index, line) in file_reader.lines().enumerate().take(10) {
            let dictionary_entry = parse_line(&line?)?;
            results.push(dictionary_entry);
        }

        //let _ = wiktionary_en_lua::intercept_witkionary_result(&results)?;
        //let entries = wiktionary_en_lua::intercept_witkionary_result(&results)?;
        //for entry in entries {
        //    println!("{}", entry.to_pretty_string());
        //}
        return Ok(());
    }

    #[traced_test]
    #[test]
    fn test_format_several() -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut results = Vec::new();

        for (_index, line) in file_reader.lines().enumerate().take(10) {
            let dictionary_entry = parse_line(&line?)?;
            results.push(dictionary_entry);
        }

        let config_handler = wiktionary_en_lua::ConfigHandler::init()?;
        let formatted_entries = config_handler.format_wiktionary_result(&results)?;
        if let Some(formatted_entries) = formatted_entries {
            for formatted_entry in formatted_entries {
                println!("{}", formatted_entry);
            }
        }
        return Ok(());
    }
}
