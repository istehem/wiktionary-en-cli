#[cfg(test)]
mod tests {
    use anyhow::{bail, Result};
    use rstest::rstest;
    use std::path::PathBuf;
    use utilities::file_utils;
    use utilities::language::Language;
    use wiktionary_en_db::couchdb_client::DbClient;

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
    async fn create_analytics() -> Result<()> {
        // use with "curl  http://<user>:<password>@localhost:5984/en/_design/analytics/_view/word_count | jq"
        let client = DbClient::init(utilities::language::Language::EN).await?;
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
    async fn word_document_count() -> Result<()> {
        // use with "curl  http://<user>:<password>@localhost:5984/en/_design/analytics/_view/word_count | jq"
        let client = DbClient::init(utilities::language::Language::EN).await?;
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
