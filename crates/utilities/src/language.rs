use self::Language::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Copy, Clone, PartialEq, Default, Serialize, Deserialize, Debug)]
pub enum Language {
    #[default]
    #[serde(rename = "en")]
    EN,
    #[serde(rename = "de")]
    DE,
    #[serde(rename = "fr")]
    FR,
    #[serde(rename = "es")]
    ES,
    #[serde(rename = "sv")]
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
        Language::iterator().map(|lang| lang.value()).collect()
    }
    pub fn from_str_or_default(language: &str) -> Language {
        language.parse().unwrap_or_default()
    }
}

impl FromStr for Language {
    type Err = anyhow::Error;

    fn from_str(language: &str) -> anyhow::Result<Self> {
        Language::iterator()
            .find(|l| l.value() == *language)
            .ok_or_else(|| anyhow::anyhow!("unsupported language code: '{}'", language))
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}
