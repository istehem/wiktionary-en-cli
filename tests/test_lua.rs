#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tracing::info;
    use tracing_test::traced_test;
    use utilities::language::*;

    use wiktionary_en_lua;

    #[traced_test]
    #[test]
    fn test_function() -> Result<()> {
        let result = wiktionary_en_lua::do_one_plus_one()?;
        assert!(result == 2);
        return Ok(());
    }

    #[traced_test]
    #[test]
    fn test_load_config() -> Result<()> {
        let config = wiktionary_en_lua::do_load_config()?;
        info!("lua returns a config with message: {}", &config.message);
        assert!(config.message == "Hello World!");
        assert!(config.language == Language::SV);
        return Ok(());
    }
}
