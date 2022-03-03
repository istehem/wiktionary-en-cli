use serde::{Deserialize, Serialize};
use self::Language::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct DictionaryEntry {
    pub lang_code : String,
    #[serde(default)]
    pub word : String,
    pub senses : Vec<Sense>,
    pub pos : String,
    #[serde(default)]
    pub translations : Vec<Translation>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Sense {
    #[serde(default)]
    pub glosses : Vec<String>,
    #[serde(default)]
    pub examples : Vec<Example>
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Example {
    #[serde(alias = "ref")]
    #[serde(default)]
    reference : Option<String>,
    pub text : String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Translation {
    pub lang : String,
    #[serde(default)]
    pub code : Option<String>,
    #[serde(default)]
    pub word : Option<String>,
}

#[derive(Copy, Clone)]
pub enum Language {
    EN,
    DE,
    FR,
    ES,
    SV
}

impl Language {
    pub fn value(&self) -> String {
        match self {
            EN => "en".to_string(),
            DE => "de".to_string(),
            FR => "fr".to_string(),
            ES => "es".to_string(),
            SV => "sv".to_string()
        }
    }
    pub fn iterator() -> impl Iterator<Item = Language> {
        [EN, DE, FR, ES, SV].iter().copied()
    }
}

pub fn parse(line : &String) -> Result<DictionaryEntry, serde_json::Error> {
    return serde_json::from_str(line);
}
