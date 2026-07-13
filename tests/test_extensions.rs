#[cfg(test)]
mod tests {
    use anyhow::{Context, Error, Result};
    use rstest::{fixture, rstest};
    use rustainers::images::GenericImage;
    use rustainers::runner::{RunOption, Runner};
    use rustainers::Container;
    use rustainers::{ImageName, WaitStrategy};
    use std::collections::HashSet;
    use std::env;
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::path::PathBuf;
    use tokio::time::{sleep, Duration};
    use utilities::file_utils;
    use utilities::language::Language;
    use wiktionary_en_db::couchdb_client::{DbClient, DbClientMutex};
    use wiktionary_en_entities::dictionary_entry::DictionaryEntry;
    use wiktionary_en_entities::result::{DictionaryResult, DidYouMean};
    use wiktionary_en_lua::extension::{ExtensionErrorType, ExtensionHandler, ExtensionResult};

    const ITERATIONS: usize = 301;
    const COUCH_DB_PORT: u16 = 5984;

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
    type CouchDBContainer = Container<GenericImage>;

    struct TestSetup {
        extension_handler: ExtensionHandler,
        // The reference to the container must not go out of scope; that would shut down the container.
        #[allow(dead_code)]
        couchdb_container: CouchDBContainer,
    }

    fn language() -> Language {
        Language::EN
    }

    fn parse_line(line: &str) -> Result<DictionaryEntry> {
        line.parse()
            .with_context(|| "Couldn't parse line in DB file.".to_string())
    }

    fn error_chain_as_strings(error: &Error) -> Vec<String> {
        error.chain().map(|e| e.to_string()).collect()
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

    #[fixture]
    async fn start_couchdb() -> CouchDBContainer {
        let name = ImageName::new_with_tag("docker.io/couchdb", "3.5.2");
        let mut image = GenericImage::new(name);
        image.add_env_var("COUCHDB_PASSWORD", env!("COUCH_DB_PASSWORD"));
        image.add_env_var("COUCHDB_USER", env!("COUCH_DB_USER"));
        image.add_port_mapping(COUCH_DB_PORT);
        image.set_wait_strategy(WaitStrategy::HttpSuccess {
            path: "/_up".to_string(),
            container_port: COUCH_DB_PORT.into(),
            https: false,
            require_valid_certs: false,
        });

        let run_option = RunOption::builder().with_remove(true).build();
        let runner = Runner::podman().unwrap();
        let container = runner.start_with_options(image, run_option).await.unwrap();

        // couchdb /up endpoint returns ok before users are initialized; this may cause 401.
        sleep(Duration::from_millis(2000)).await;
        container
    }

    #[fixture]
    async fn test_setup(
        #[from(start_couchdb)]
        #[future]
        couchdb_container: Container<GenericImage>,
    ) -> TestSetup {
        let container = couchdb_container.await;
        let port = container.host_port(COUCH_DB_PORT).await.unwrap();
        unsafe {
            env::set_var("COUCH_DB_HOST", format!("http://localhost:{}", port));
        }
        let db_client = DbClient::init(language()).await.unwrap();
        TestSetup {
            extension_handler: ExtensionHandler::init(DbClientMutex::from(db_client))
                .await
                .unwrap(),
            couchdb_container: container,
        }
    }

    #[rstest]
    #[tokio::test]
    async fn format_dictionary_entries(
        #[from(test_setup)]
        #[future]
        test_setup: TestSetup,
    ) -> Result<()> {
        let db_path = PathBuf::from(utilities::DICTIONARY_DB_PATH!(language()));
        let file_reader: BufReader<File> = file_utils::get_file_reader(&db_path)?;
        let mut results = Vec::new();

        for (_index, line) in file_reader.lines().enumerate().take(10) {
            let dictionary_entry = parse_line(&line?)?;
            results.push(dictionary_entry);
        }

        let awaited_test_setup = test_setup.await;
        let formatted_entries = awaited_test_setup
            .extension_handler
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
        #[from(test_setup)]
        #[future]
        test_setup: TestSetup,
    ) -> Result<()> {
        let awaited_test_setup = test_setup.await;
        let formatted_banner = awaited_test_setup
            .extension_handler
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
    async fn format_history_entries(
        #[from(test_setup)]
        #[future]
        test_setup: TestSetup,
    ) -> Result<()> {
        let awaited_test_setup = test_setup.await;
        let extension_handler = awaited_test_setup.extension_handler;

        intercept_dictionary_entries(&extension_handler).await?;
        let extension_result: ExtensionResult<String> =
            extension_handler.call_extension("history", &vec![]).await?;

        println!("{}", extension_result.result);

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn delete_history_entries(
        #[from(test_setup)]
        #[future]
        test_setup: TestSetup,
    ) -> Result<()> {
        let awaited_test_setup = test_setup.await;
        let extension_handler = awaited_test_setup.extension_handler;

        let size = intercept_dictionary_entries(&extension_handler).await?;
        let extension_result: ExtensionResult<String> = extension_handler
            .call_extension("history", &vec!["delete".to_string()])
            .await?;

        assert_contains!(extension_result.result, &format!("{}", size));

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn count_history_entries(
        #[from(test_setup)]
        #[future]
        test_setup: TestSetup,
    ) -> Result<()> {
        let awaited_test_setup = test_setup.await;
        let extension_handler = awaited_test_setup.extension_handler;

        let size = intercept_dictionary_entries(&extension_handler).await?;
        let history_count: ExtensionResult<usize> = extension_handler
            .call_extension("history", &vec!["count".to_string()])
            .await?;

        assert_eq!(size, history_count.result);

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn history_with_unknown_option(
        #[from(test_setup)]
        #[future]
        test_setup: TestSetup,
    ) -> Result<()> {
        let awaited_test_setup = test_setup.await;
        let extension_handler = awaited_test_setup.extension_handler;

        let result: Result<ExtensionResult<String>> = extension_handler
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
        #[from(test_setup)]
        #[future]
        test_setup: TestSetup,
    ) -> Result<()> {
        let awaited_test_setup = test_setup.await;
        let extension_handler = awaited_test_setup.extension_handler;

        let extension_name = "unknown";
        let result: Result<ExtensionResult<String>> = extension_handler
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
        #[from(test_setup)]
        #[future]
        test_setup: TestSetup,
        #[case] extension_name: &str,
    ) -> Result<()> {
        let awaited_test_setup = test_setup.await;
        let extension_handler = awaited_test_setup.extension_handler;

        let result: Result<ExtensionResult<String>> = extension_handler
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
