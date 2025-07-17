#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tracing::info;
    use tracing_test::traced_test;
    use wiktionary_en_lua;

    #[traced_test]
    #[test]
    fn test_load_config() -> Result<()> {
        let config_handler = wiktionary_en_lua::ConfigHandler::init()?;
        info!("language configured: {}", config_handler.config.language);
        return Ok(());
    }
}
