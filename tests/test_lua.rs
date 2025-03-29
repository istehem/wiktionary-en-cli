#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tracing_test::traced_test;

    use wiktionary_en_lua;

    #[traced_test]
    #[test]
    fn test_function() -> Result<()> {
        let result = wiktionary_en_lua::do_one_plus_one()?;
        assert!(result == 2);
        return Ok(());
    }
}
