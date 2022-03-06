use serde::{Deserialize, Serialize};
use indoc::{formatdoc};
use colored::Colorize;

pub mod language;

#[derive(Serialize, Deserialize, Clone)]
pub struct DictionaryEntry {
    lang_code : String,
    #[serde(default)]
    pub word : String,
    senses : Vec<Sense>,
    pos : String,
    #[serde(default)]
    translations : Vec<Translation>
}

#[derive(Serialize, Deserialize, Clone)]
struct Sense {
    #[serde(default)]
    glosses : Vec<String>,
    #[serde(default)]
    examples : Vec<Example>
}

#[derive(Serialize, Deserialize, Clone)]
struct Example {
    #[serde(alias = "ref")]
    #[serde(default)]
    reference : Option<String>,
    text : String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Translation {
    lang : String,
    #[serde(default)]
    code : Option<String>,
    #[serde(default)]
    word : Option<String>,
}

impl DictionaryEntry {
    pub fn to_pretty_string(&self) -> String {
    let senses : String = self.senses
        .clone()
        .into_iter()
        .enumerate()
        .fold(String::new(), |res, (_i, sense)| {
                    res + &formatdoc!("
                                {}
                                {}
                                -------------------------------------------
                                ",
                                format_glosses(&sense.glosses),
                                format_examples(&sense.examples))
        });

    return formatdoc!("
              -------------------------------------------
              {} ({})
              -------------------------------------------
              {}
              {}
              -------------------------------------------
              ", &self.word.clone().green(),
                 self.pos, senses, format_translations(&self.translations));
    }
}

fn format_glosses(glosses : &Vec<String>) -> String{
    match glosses.as_slice() {
        [] => "Glossaries".to_string(),
        xs => {
           return xs.into_iter()
               .enumerate()
               .fold("Glossaries:\n".to_string(), |res, (i, gloss)| {
                    return res + &formatdoc!(" {}) {}\n", i, gloss);
                })
        }
    }
}

fn format_examples(examples : &Vec<Example>) -> String{
    match examples.as_slice() {
        [] => "Examples:".to_string(),
        xs => {
           return xs.into_iter()
               .enumerate()
               .fold("Examples:\n".to_string(), |res, (i, example)| {
                    return res + &formatdoc!(" {}) {}\n", i, example.text);
                })
        }
    }
}

fn format_translations(translations : &Vec<Translation>) -> String {
    match translations.as_slice() {
        [] => "Translations:".to_string(),
        _  => {
            let langs : Vec<Option<String>> = language::Language::iterator()
                .map(|lang| { Some(lang.value()) })
                .collect();
            let mut filtered_translations : Vec<Translation> = translations.clone()
                .into_iter()
                .filter(|translation| { langs.contains(&&translation.code) })
                .collect();
            filtered_translations.sort_by(|t1, t2| t1.lang.cmp(&t2.lang));
            return filtered_translations.into_iter()
               .fold("Translations:\n".to_string(), |res, translation| {
                    return res + &formatdoc!(" {}) {}\n",
                         translation.lang,
                         translation.word.clone().unwrap_or_else(String::new));
                })
        }
    }
}

pub fn parse(line : &String) -> Result<DictionaryEntry, serde_json::Error> {
    return serde_json::from_str(line);
}
