use anyhow::Result;
use utilities::language::*;

#[derive(Default, Clone)]
pub struct Config {
    pub language: Language,
}

impl Config {
    pub fn parse_language_or_use_config_or_default(
        &self,
        language: &Option<String>,
    ) -> Result<Language> {
        if let Some(language) = language {
            return language.parse();
        }
        Ok(self.language)
    }
}
