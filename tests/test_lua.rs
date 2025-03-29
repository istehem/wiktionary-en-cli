#[cfg(test)]
mod tests {
    use anyhow::Result;
    use tracing_test::traced_test;

    use wiktionary_en_lua;

    #[traced_test]
    #[test]
    fn can_load_some_lua_code() -> Result<()> {
        wiktionary_en_lua::hello()?;
        Ok(())
    }
}
