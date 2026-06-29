#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rstest::rstest;
    use wiktionary_en_db::couchdb_client::DbClient;

    #[rstest]
    #[tokio::test]
    async fn download_a_file() -> Result<()> {
        DbClient::init(utilities::language::Language::EN).await?;
        Ok(())
    }
}
