use self::Language::*;
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq)]
pub enum Language {
    EN,
    DE,
    FR,
    ES,
    SV,
}

impl Language {
    pub fn value(&self) -> String {
        match self {
            EN => "en".to_string(),
            DE => "de".to_string(),
            FR => "fr".to_string(),
            ES => "es".to_string(),
            SV => "sv".to_string(),
        }
    }
    pub fn iterator() -> impl Iterator<Item = Language> {
        [EN, DE, FR, ES, SV].iter().copied()
    }
    pub fn as_strings() -> Vec<String> {
        return Language::iterator().map(|lang| lang.value()).collect();
    }
    pub fn from_str_or_default(language: &str) -> Language {
        return language.parse().unwrap_or_default();
    }
}

impl Default for Language {
    fn default() -> Self {
        EN
    }
}

impl FromStr for Language {
    type Err = anyhow::Error;

    fn from_str(language: &str) -> anyhow::Result<Self> {
        return Language::iterator()
            .find(|l| l.value() == String::from(language))
            .ok_or_else(|| anyhow::anyhow!("unsupported language code: '{}'", language));
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(f, "{}", self.value());
    }
}
