#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tracing::info;
    use tracing_test::traced_test;
    use wiktionary_en_lua::config::ConfigHandler;

    #[traced_test]
    #[test]
    fn test_load_config() -> Result<()> {
        let config_handler = ConfigHandler::init()?;
        info!("language configured: {}", config_handler.config.language);
        Ok(())
    }
}
