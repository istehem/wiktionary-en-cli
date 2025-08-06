#[cfg(test)]
mod tests {
    use anyhow::{Context, Error, Result};
    use lazy_static::lazy_static;
    use rstest::*;
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use utilities::file_utils;
    use utilities::language::Language;
    use wiktionary_en_db::client::{DbClient, DbClientMutex};
    use wiktionary_en_entities::dictionary_entry::DictionaryEntry;
    use wiktionary_en_entities::result::{DictionaryResult, DidYouMean, ExtensionErrorType};
    use wiktionary_en_lua::ExtensionHandler;

    const ITERATIONS: usize = 100;

    lazy_static! {
        static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
    }

    macro_rules! assert_contains {
        ($haystack:expr, $needle:expr) => {
            assert!(
                $haystack.contains($needle),
                "the string {:?} does not contain {:?}",
                $haystack,
                $needle
            )
        };
    }

    fn language() -> Language {
        Language::EN
    }

    fn intercept_dictionary_entries(extension_handler: &ExtensionHandler) -> Result<usize> {
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;

        let mut found_words = HashSet::new();

        for (_index, line) in file_reader.lines().enumerate().take(ITERATIONS) {
            let dictionary_entry = parse_line(&line?)?;
            let mut dictionary_result = DictionaryResult {
                hits: vec![dictionary_entry.clone()],
                did_you_mean: None,
                word: dictionary_entry.word,
            };
            if !found_words.contains(&dictionary_result.word) {
                extension_handler.intercept_dictionary_result(&mut dictionary_result)?;
            }
            found_words.insert(dictionary_result.word);
        }
        Ok(found_words.len())
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

    fn parse_line(line: &str) -> Result<DictionaryEntry> {
        line.parse()
            .with_context(|| "Couldn't parse line in DB file.".to_string())
    }

    #[rstest]
    fn test_format_dictionary_entries(
        #[from(shared_extension_handler)] extension_handler: ExtensionHandler,
    ) -> Result<()> {
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language()));
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

    #[rstest]
    fn test_format_history_entries(
        #[from(shared_extension_handler)] extension_handler: ExtensionHandler,
    ) -> Result<()> {
        {
            let _guard = TEST_MUTEX.lock().unwrap();
            intercept_dictionary_entries(&extension_handler)?;
        }
        let extension_result = extension_handler.call_extension("history", &vec![])?;
        println!("{}", extension_result.result);

        Ok(())
    }

    #[rstest]
    fn test_delete_history_entries(
        #[from(shared_extension_handler)] extension_handler: ExtensionHandler,
    ) -> Result<()> {
        let _guard = TEST_MUTEX.lock().unwrap();

        let call_delete =
            || extension_handler.call_extension("history", &vec!["delete".to_string()]);

        call_delete()?;
        let size = intercept_dictionary_entries(&extension_handler)?;
        let extension_result = call_delete()?;

        assert_contains!(extension_result.result, &format!("{}", size));

        Ok(())
    }

    fn error_chain_as_strings(error: &Error) -> Vec<String> {
        error.chain().map(|e| e.to_string()).collect()
    }

    #[rstest]
    fn test_history_with_unknown_argument(
        #[from(shared_extension_handler)] extension_handler: ExtensionHandler,
    ) -> Result<()> {
        let error = extension_handler
            .call_extension("history", &vec!["unknown".to_string()])
            .unwrap_err();
        assert_contains!(
            error_chain_as_strings(&error),
            &ExtensionErrorType::UnknownOption.to_string()
        );

        Ok(())
    }

    #[rstest]
    fn test_calling_unknown_extension(
        #[from(shared_extension_handler)] extension_handler: ExtensionHandler,
    ) -> Result<()> {
        let extension_name = "unknown";
        let error = extension_handler
            .call_extension(extension_name, &vec![])
            .unwrap_err();
        assert_contains!(
            error_chain_as_strings(&error),
            &format!("extension '{}' not found", extension_name)
        );

        Ok(())
    }
}
