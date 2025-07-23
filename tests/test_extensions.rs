#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::path::PathBuf;
    use tracing_test::traced_test;
    use utilities::file_utils;
    use utilities::language::Language;
    use wiktionary_en_db::client::{DbClient, DbClientMutex};
    use wiktionary_en_entities::dictionary_entry::DictionaryEntry;
    use wiktionary_en_entities::result::DictionaryResult;

    use wiktionary_en_lua;

    use rstest::*;

    #[fixture]
    #[once]
    fn shared_db_client() -> DbClientMutex {
        let language = Language::EN;
        let db_client = DbClient::init(language).unwrap();
        DbClientMutex::from(db_client)
    }

    fn parse_line(line: &String) -> Result<DictionaryEntry> {
        line.parse()
            .with_context(|| format!("{}", "Couldn't parse line in DB file."))
    }

    #[traced_test]
    #[rstest]
    fn test_intercept(shared_db_client: &DbClientMutex) -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut results = Vec::new();

        for (_index, line) in file_reader.lines().enumerate().take(10) {
            let dictionary_entry = parse_line(&line?)?;
            results.push(dictionary_entry);
        }

        let mut dictionary_result = DictionaryResult {
            hits: results,
            did_you_mean: None,
            word: "test".to_string(),
        };
        let extension_handler =
            wiktionary_en_lua::ExtensionHandler::init(shared_db_client.clone())?;
        extension_handler.intercept_dictionary_result(&mut dictionary_result)?;
        for entry in dictionary_result.hits {
            println!("{}", entry);
        }
        return Ok(());
    }

    #[traced_test]
    #[rstest]
    fn test_format(shared_db_client: &DbClientMutex) -> Result<()> {
        let language = Language::EN;
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut results = Vec::new();

        for (_index, line) in file_reader.lines().enumerate().take(10) {
            let dictionary_entry = parse_line(&line?)?;
            results.push(dictionary_entry);
        }

        let extension_handler =
            wiktionary_en_lua::ExtensionHandler::init(shared_db_client.clone())?;
        let formatted_entries = extension_handler.format_wiktionary_entries(&results)?;
        if let Some(formatted_entries) = formatted_entries {
            for formatted_entry in formatted_entries {
                println!("{}", formatted_entry);
            }
        }
        return Ok(());
    }
}
