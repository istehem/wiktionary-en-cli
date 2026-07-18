#[cfg(test)]
mod tests {
    use anyhow::{bail, Result};
    use rstest::{fixture, rstest};
    use std::env;
    use std::path::PathBuf;
    use utilities::file_utils;
    use utilities::language::Language;
    use wiktionary_en_db::client::DbClient;

    mod common {
        include!("common/couchdb_container.rs");
    }
    use common::CouchDBContainer;

    struct TestSetup {
        db_client: DbClient,
        // The reference to the container must not go out of scope; that would shut down the container.
        #[allow(dead_code)]
        couchdb_container: CouchDBContainer,
    }

    #[fixture]
    async fn start_couchdb() -> CouchDBContainer {
        common::start_couchdb().await.unwrap()
    }

    #[fixture]
    async fn test_setup(
        #[from(start_couchdb)]
        #[future]
        couchdb_container: CouchDBContainer,
    ) -> TestSetup {
        let container = couchdb_container.await;
        let port = container.host_port(common::COUCH_DB_PORT).await.unwrap();
        unsafe {
            env::set_var("COUCH_DB_HOST", format!("http://localhost:{}", port));
        }
        let db_client = DbClient::init(Language::EN).await.unwrap();
        TestSetup {
            db_client,
            couchdb_container: container,
        }
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "ignore for now"]
    async fn find_by_word() -> Result<()> {
        let client = DbClient::init(utilities::language::Language::EN).await?;
        let entries = client.find_by_word("soccer").await?;
        for entry in entries {
            print!("{}", entry.to_pretty_string());
        }
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_analytics(
        #[from(test_setup)]
        #[future]
        test_setup: TestSetup,
    ) -> Result<()> {
        let awaited_test_setup = test_setup.await;
        // use with "curl  http://<user>:<password>@localhost:5984/en/_design/analytics/_view/word_count | jq"
        let container = awaited_test_setup.couchdb_container;
        let port = container.host_port(common::COUCH_DB_PORT).await.unwrap();
        unsafe {
            env::set_var("COUCH_DB_HOST", format!("http://localhost:{}", port));
        }
        let client = DbClient::init(Language::EN).await.unwrap();
        let result = client.create_analytics().await?;
        if result {
            println!("created an analytics design document");
        } else {
            println!("an analytics design document already exists");
        }
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn word_document_count(
        #[from(test_setup)]
        #[future]
        test_setup: TestSetup,
    ) -> Result<()> {
        let awaited_test_setup = test_setup.await;
        let client = awaited_test_setup.db_client;
        client.create_analytics().await?;
        let result = client.word_document_count().await?;
        println!("counted {} number of words", result);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "db already populated"]
    async fn find_insert_file() -> Result<()> {
        let mut client = DbClient::init(Language::EN).await?;

        let db_path: PathBuf = file_utils::get_db_path(None, &Language::EN);
        match file_utils::get_file_reader(&db_path) {
            Ok(path) => client.insert_wiktionary_file(path, false).await?,
            Err(err) => bail!(err),
        };
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    #[ignore = "index already created"]
    async fn create_index_on_word() -> Result<()> {
        let client = DbClient::init(Language::EN).await?;
        assert!(client.create_index_on_word().await?);
        Ok(())
    }
}
