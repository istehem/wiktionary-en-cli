#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rstest::rstest;
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
}
