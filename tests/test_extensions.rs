#[cfg(test)]
mod tests {
    use anyhow::{bail, Context, Result};
    use rstest::*;
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::path::PathBuf;
    use std::sync::MutexGuard;
    use utilities::file_utils;
    use utilities::language::Language;
    use wiktionary_en_db::client::{DbClient, DbClientMutex};
    use wiktionary_en_entities::dictionary_entry::DictionaryEntry;
    use wiktionary_en_entities::result::{DictionaryResult, DidYouMean};
    use wiktionary_en_lua::ExtensionHandler;

    fn language() -> Language {
        Language::EN
    }

    #[fixture]
    #[once]
    fn shared_db_client() -> DbClientMutex {
        let db_client = DbClient::init(language()).unwrap();
        DbClientMutex::from(db_client)
    }

    #[fixture]
    fn shared_extension_handler(
        #[from(shared_db_client)] db_client: &DbClientMutex,
    ) -> ExtensionHandler {
        ExtensionHandler::init(db_client.clone()).unwrap()
    }

    fn parse_line(line: &String) -> Result<DictionaryEntry> {
        line.parse()
            .with_context(|| format!("{}", "Couldn't parse line in DB file."))
    }

    #[rstest]
    fn test_interception(
        #[from(shared_extension_handler)] extension_handler: ExtensionHandler,
    ) -> Result<()> {
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language().to_string()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;

        for (_index, line) in file_reader.lines().enumerate().take(10) {
            let dictionary_entry = parse_line(&line?)?;
            let mut dictionary_result = DictionaryResult {
                hits: vec![dictionary_entry.clone()],
                did_you_mean: None,
                word: dictionary_entry.word,
            };
            extension_handler.intercept_dictionary_result(&mut dictionary_result)?;
            for entry in dictionary_result.hits {
                println!("{}", entry);
            }
        }

        Ok(())
    }

    #[rstest]
    fn test_format_dictionary_entries(
        #[from(shared_extension_handler)] extension_handler: ExtensionHandler,
    ) -> Result<()> {
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language().to_string()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut results = Vec::new();

        for (_index, line) in file_reader.lines().enumerate().take(10) {
            let dictionary_entry = parse_line(&line?)?;
            results.push(dictionary_entry);
        }

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
        #[from(shared_extension_handler)] extension_handler: ExtensionHandler,
    ) -> Result<()> {
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

    fn lock(db_client: &DbClientMutex) -> Result<MutexGuard<'_, DbClient>> {
        match db_client.client.lock() {
            Ok(db_client) => Ok(db_client),
            Err(err) => bail!(err.to_string()),
        }
    }

    #[rstest]
    fn test_format_history_entries(
        #[from(shared_db_client)] db_client: &DbClientMutex,
        #[from(shared_extension_handler)] extension_handler: ExtensionHandler,
    ) -> Result<()> {
        let history_entries = lock(&db_client)?.find_all_in_history()?;
        let formatted_entries = extension_handler.format_history_entries(&history_entries)?;

        if let Some(formatted_entries) = formatted_entries {
            for formatted_entry in formatted_entries {
                println!("{}", formatted_entry);
            }
        }

        Ok(())
    }
}
