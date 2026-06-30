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
    async fn find_by_word() -> Result<()> {
        let client = DbClient::init(utilities::language::Language::EN).await?;
        let entries = client.find_by_word("test").await?;
        for entry in entries {
            print!("{}", entry.to_pretty_string());
        }
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn find_insert_file() -> Result<()> {
        let client = DbClient::init(Language::EN).await?;

        let db_path: PathBuf = file_utils::get_db_path(None, &Language::EN);
        match file_utils::get_file_reader(&db_path) {
            Ok(path) => client.insert_wiktionary_file(path, false).await?,
            Err(err) => bail!(err),
        };
        Ok(())
    }
}
