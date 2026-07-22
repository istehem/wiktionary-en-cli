#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rstest::{fixture, rstest};
    use std::env;
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

    async fn insert_test_data(client: &mut DbClient) -> Result<usize> {
        let path = file_utils::get_db_path(
            Some("./data/wiktionary-en-test.jsonl".to_string()),
            &Language::EN,
        );
        client
            .insert_wiktionary_file(file_utils::get_file_reader(&path)?, false)
            .await
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
    async fn find_by_word(
        #[from(test_setup)]
        #[future]
        test_setup: TestSetup,
    ) -> Result<()> {
        let awaited_test_setup = test_setup.await;
        let mut client = awaited_test_setup.db_client;
        client.create_analytics().await?;
        insert_test_data(&mut client).await?;
        let entries = client.find_by_word("dictionary").await?;
        assert_eq!(entries.len(), 1);
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
        let client = awaited_test_setup.db_client;
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
        let mut client = awaited_test_setup.db_client;
        client.create_analytics().await?;
        insert_test_data(&mut client).await?;
        let result = client.word_document_count().await?;
        assert_eq!(result, 1);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_insert_file(
        #[from(test_setup)]
        #[future]
        test_setup: TestSetup,
    ) -> Result<()> {
        let awaited_test_setup = test_setup.await;
        let mut client = awaited_test_setup.db_client;
        client.create_analytics().await?;
        let entries = insert_test_data(&mut client).await?;
        assert_eq!(entries, 1);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn create_index_on_word(
        #[from(test_setup)]
        #[future]
        test_setup: TestSetup,
    ) -> Result<()> {
        let awaited_test_client = test_setup.await;
        let client = awaited_test_client.db_client;
        assert!(client.create_index_on_word().await?);
        Ok(())
    }
}
