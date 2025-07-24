use utilities::language::Language;

#[derive(Default, Clone)]
pub struct Config {
    pub language: Language,
}

impl Config {
    pub fn or_use_config_or_default(&self, language: Option<Language>) -> Language {
        if let Some(language) = language {
            return language;
        }
        self.language
    }
}
