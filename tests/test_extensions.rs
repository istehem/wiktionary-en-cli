#[cfg(test)]
mod tests {
    use anyhow::{Context, Error, Result};
    use rstest::{fixture, rstest};
    use serial_test::serial;
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::path::PathBuf;
    use utilities::file_utils;
    use utilities::language::Language;
    use wiktionary_en_db::couchdb_client::{DbClient, DbClientMutex};
    use wiktionary_en_entities::dictionary_entry::DictionaryEntry;
    use wiktionary_en_entities::result::{DictionaryResult, DidYouMean};
    use wiktionary_en_lua::extension::{ExtensionErrorType, ExtensionHandler, ExtensionResult};

    const ITERATIONS: usize = 100;

    /*
    use lazy_static::lazy_static;
    use std::sync::Mutex;
    lazy_static! {
        static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
    }
    */

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

    macro_rules! directly_calling_inner_workings_extension_error_msg {
        ($extension_name:expr) => {
            format!(
                "the extension '{}' is used for inner workings only and can't be called directly",
                $extension_name
            )
        };
    }

    fn language() -> Language {
        Language::EN
    }

    async fn intercept_dictionary_entries(extension_handler: &ExtensionHandler) -> Result<usize> {
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
                extension_handler
                    .intercept_dictionary_result(&mut dictionary_result)
                    .await?;
            }
            found_words.insert(dictionary_result.word);
        }
        Ok(found_words.len())
    }

    //#[once]
    #[fixture]
    async fn shared_db_client() -> DbClientMutex {
        //let rt = Runtime::new().unwrap();
        //rt.block_on(async {
        //    let db_client = DbClient::init(language()).await.unwrap();
        //    DbClientMutex::from(db_client)
        //})
        let db_client = DbClient::init(language()).await.unwrap();
        DbClientMutex::from(db_client)
    }

    #[fixture]
    async fn shared_extension_handler(
        #[from(shared_db_client)]
        #[future]
        db_client: DbClientMutex,
    ) -> ExtensionHandler {
        ExtensionHandler::init(db_client.await).await.unwrap()
    }

    fn parse_line(line: &str) -> Result<DictionaryEntry> {
        line.parse()
            .with_context(|| "Couldn't parse line in DB file.".to_string())
    }

    #[rstest]
    #[tokio::test]
    async fn format_dictionary_entries(
        #[from(shared_extension_handler)]
        #[future]
        extension_handler: ExtensionHandler,
    ) -> Result<()> {
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut results = Vec::new();

        for (_index, line) in file_reader.lines().enumerate().take(10) {
            let dictionary_entry = parse_line(&line?)?;
            results.push(dictionary_entry);
        }

        let formatted_entries = extension_handler
            .await
            .format_dictionary_entries(&results)
            .await?;
        if let Some(formatted_entries) = formatted_entries {
            for formatted_entry in formatted_entries {
                println!("{}", formatted_entry);
            }
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn format_did_you_mean_banner(
        #[from(shared_extension_handler)]
        #[future]
        extension_handler: ExtensionHandler,
    ) -> Result<()> {
        let formatted_banner = extension_handler
            .await
            .format_dictionary_did_you_mean_banner(&DidYouMean {
                searched_for: "You searched for".to_string(),
                suggestion: "... but probably meant".to_string(),
            })
            .await?;
        if let Some(formatted_banner) = formatted_banner {
            println!("{}", formatted_banner);
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn format_history_entries(
        #[from(shared_extension_handler)]
        #[future]
        extension_handler: ExtensionHandler,
    ) -> Result<()> {
        let awaited_extension_handler = extension_handler.await;
        intercept_dictionary_entries(&awaited_extension_handler).await?;
        let extension_result: ExtensionResult<String> = awaited_extension_handler
            .call_extension("history", &vec![])
            .await?;
        println!("{}", extension_result.result);

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn delete_history_entries(
        #[from(shared_extension_handler)]
        #[future]
        extension_handler: ExtensionHandler,
    ) -> Result<()> {
        let awaited_extension_handler = extension_handler.await;
        //let _guard = TEST_MUTEX.lock().unwrap();

        let call_delete = async || {
            awaited_extension_handler
                .call_extension("history", &vec!["delete".to_string()])
                .await
        };

        call_delete().await?;
        let size = intercept_dictionary_entries(&awaited_extension_handler).await?;
        let extension_result: ExtensionResult<String> = call_delete().await?;

        assert_contains!(extension_result.result, &format!("{}", size));

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn count_history_entries(
        #[from(shared_extension_handler)]
        #[future]
        extension_handler: ExtensionHandler,
    ) -> Result<()> {
        let awaited_extension_handler = extension_handler.await;
        //let _guard = TEST_MUTEX.lock().unwrap();
        let _: ExtensionResult<String> = awaited_extension_handler
            .call_extension("history", &vec!["delete".to_string()])
            .await?;
        let size = intercept_dictionary_entries(&awaited_extension_handler).await?;
        let history_count: ExtensionResult<usize> = awaited_extension_handler
            .call_extension("history", &vec!["count".to_string()])
            .await?;

        assert_eq!(size, history_count.result);

        Ok(())
    }

    fn error_chain_as_strings(error: &Error) -> Vec<String> {
        error.chain().map(|e| e.to_string()).collect()
    }

    #[rstest]
    #[tokio::test]
    async fn history_with_unknown_option(
        #[from(shared_extension_handler)]
        #[future]
        extension_handler: ExtensionHandler,
    ) -> Result<()> {
        let awaited_extension_handler = extension_handler.await;
        let result: Result<ExtensionResult<String>> = awaited_extension_handler
            .call_extension("history", &vec!["unknown".to_string()])
            .await;
        let error = result.unwrap_err();
        assert_contains!(
            error_chain_as_strings(&error),
            &ExtensionErrorType::UnknownOption.to_string()
        );

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn calling_unknown_extension(
        #[from(shared_extension_handler)]
        #[future]
        extension_handler: ExtensionHandler,
    ) -> Result<()> {
        let awaited_extension_handler = extension_handler.await;
        let extension_name = "unknown";
        let result: Result<ExtensionResult<String>> = awaited_extension_handler
            .call_extension(extension_name, &vec![])
            .await;
        let error = result.unwrap_err();
        assert_contains!(
            error_chain_as_strings(&error),
            &format!("extension '{}' not found", extension_name)
        );

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[case::intercept("intercept")]
    #[case::format_entry("format_entry")]
    #[case::format_did_you_mean_banner("format_did_you_mean_banner")]
    async fn directly_calling_inner_workings_extension(
        #[from(shared_extension_handler)]
        #[future]
        extension_handler: ExtensionHandler,
        #[case] extension_name: &str,
    ) -> Result<()> {
        let awaited_extension_handler = extension_handler.await;
        let result: Result<ExtensionResult<String>> = awaited_extension_handler
            .call_extension(extension_name, &vec![])
            .await;
        let error = result.unwrap_err();
        assert_contains!(
            error_chain_as_strings(&error),
            &directly_calling_inner_workings_extension_error_msg!(extension_name)
        );

        Ok(())
    }
}
