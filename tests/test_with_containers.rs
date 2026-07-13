#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use rstest::{fixture, rstest};
    use rustainers::images::GenericImage;
    use rustainers::runner::Runner;
    use rustainers::Container;
    use rustainers::{HealthCheck, ImageName, WaitStrategy};
    use serial_test::serial;
    use std::collections::HashSet;
    use std::env;
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::path::PathBuf;
    use utilities::file_utils;
    use utilities::language::Language;
    use wiktionary_en_db::couchdb_client::{DbClient, DbClientMutex};
    use wiktionary_en_entities::dictionary_entry::DictionaryEntry;
    use wiktionary_en_entities::result::DictionaryResult;
    use wiktionary_en_lua::extension::{ExtensionHandler, ExtensionResult};

    const ITERATIONS: usize = 301;
    const COUCH_DB_PORT: u16 = 5984;

    type CouchDBContainer = Container<GenericImage>;

    struct TestSetup {
        extension_handler: ExtensionHandler,
        couchdb_container: CouchDBContainer,
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

    #[fixture]
    async fn start_couchdb() -> CouchDBContainer {
        let name = ImageName::new_with_tag("docker.io/couchdb", "3.5.2");
        let mut image = GenericImage::new(name);
        image.add_env_var("COUCHDB_PASSWORD", env!("COUCH_DB_PASSWORD"));
        image.add_env_var("COUCHDB_USER", env!("COUCH_DB_USER"));
        image.add_port_mapping(COUCH_DB_PORT);
        let health_check = HealthCheck::builder()
            .with_command(format!(
                "bash -c 'echo > /dev/tcp/127.0.0.1/{}'",
                COUCH_DB_PORT
            ))
            .build();
        image.set_wait_strategy(WaitStrategy::custom_health_check(health_check));

        let runner = Runner::auto().unwrap();
        let container = runner.start(image).await.unwrap();

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
        env::set_var("COUCH_DB_HOST", format!("http://localhost:{}", port));
        let db_client = DbClient::init(language()).await.unwrap();
        TestSetup {
            extension_handler: ExtensionHandler::init(DbClientMutex::from(db_client))
                .await
                .unwrap(),
            couchdb_container: container,
        }
    }

    fn parse_line(line: &str) -> Result<DictionaryEntry> {
        line.parse()
            .with_context(|| "Couldn't parse line in DB file.".to_string())
    }

    #[rstest]
    #[tokio::test]
    #[serial]
    async fn count_history_entries(
        #[from(test_setup)]
        #[future]
        test_setup: TestSetup,
    ) -> Result<()> {
        let awaited_test_setup = test_setup.await;
        let _container = awaited_test_setup.couchdb_container;
        let awaited_extension_handler = awaited_test_setup.extension_handler;
        let size = intercept_dictionary_entries(&awaited_extension_handler).await?;
        let history_count: ExtensionResult<usize> = awaited_extension_handler
            .call_extension("history", &vec!["count".to_string()])
            .await?;

        assert_eq!(size, history_count.result);

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore]
    async fn start_containers() -> Result<()> {
        start_couchdb().await;
        Ok(())
    }
}
