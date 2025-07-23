#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::path::PathBuf;
    use utilities::file_utils;
    use utilities::language::Language;
    use wiktionary_en_db::client::{DbClient, DbClientMutex};
    use wiktionary_en_entities::dictionary_entry::DictionaryEntry;
    use wiktionary_en_entities::result::{DictionaryResult, DidYouMean};

    use wiktionary_en_lua;

    use rstest::*;

    fn language() -> Language {
        Language::EN
    }

    #[fixture]
    #[once]
    fn shared_db_client() -> DbClientMutex {
        let db_client = DbClient::init(language()).unwrap();
        DbClientMutex::from(db_client)
    }

    fn parse_line(line: &String) -> Result<DictionaryEntry> {
        line.parse()
            .with_context(|| format!("{}", "Couldn't parse line in DB file."))
    }

    #[rstest]
    fn test_intercept(#[from(shared_db_client)] db_client: &DbClientMutex) -> Result<()> {
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language().to_string()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;

        for (_index, line) in file_reader.lines().enumerate().take(10) {
            let dictionary_entry = parse_line(&line?)?;
            let mut dictionary_result = DictionaryResult {
                hits: vec![dictionary_entry.clone()],
                did_you_mean: None,
                word: dictionary_entry.word,
            };
            let extension_handler = wiktionary_en_lua::ExtensionHandler::init(db_client.clone())?;
            extension_handler.intercept_dictionary_result(&mut dictionary_result)?;
            for entry in dictionary_result.hits {
                println!("{}", entry);
            }
        }

        Ok(())
    }

    #[rstest]
    fn test_format_dictionary_entry(
        #[from(shared_db_client)] shared_db_client: &DbClientMutex,
    ) -> Result<()> {
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language().to_string()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut results = Vec::new();

        for (_index, line) in file_reader.lines().enumerate().take(10) {
            let dictionary_entry = parse_line(&line?)?;
            results.push(dictionary_entry);
        }

        let extension_handler =
            wiktionary_en_lua::ExtensionHandler::init(shared_db_client.clone())?;
        let formatted_entries = extension_handler.format_dictionary_entries(&results)?;
        if let Some(formatted_entries) = formatted_entries {
            for formatted_entry in formatted_entries {
                println!("{}", formatted_entry);
            }
        }
        Ok(())
    }

    #[rstest]
    fn test_format_did_you_mean_banner(
        #[from(shared_db_client)] shared_db_client: &DbClientMutex,
    ) -> Result<()> {
        let extension_handler =
            wiktionary_en_lua::ExtensionHandler::init(shared_db_client.clone())?;
        let formatted_banner =
            extension_handler.format_dictionary_did_you_mean_banner(&DidYouMean {
                searched_for: "You searched for".to_string(),
                suggestion: "... but probably meant".to_string(),
            })?;
        if let Some(formatted_banner) = formatted_banner {
            println!("{}", formatted_banner);
        }
        Ok(())
    }
}
