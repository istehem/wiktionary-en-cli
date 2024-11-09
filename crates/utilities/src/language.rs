use self::Language::*;

#[derive(Copy, Clone)]
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
    pub fn from_string(language: &String) -> Option<Language> {
        return Language::iterator().find(|l| &l.value() == language);
    }
}

impl Default for Language {
    fn default() -> Self {
        EN
    }
}
