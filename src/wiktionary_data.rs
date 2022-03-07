use serde::{Deserialize, Serialize};
use indoc::{formatdoc};
use colored::Colorize;
use anyhow::Result;
use colored::ColoredString;

pub mod language;

#[derive(Serialize, Deserialize, Clone)]
pub struct DictionaryEntry {
    lang_code : String,
    #[serde(default)]
    pub word : String,
    senses : Vec<Sense>,
    pos : String,
    #[serde(default)]
    translations : Vec<Translation>,
    #[serde(default)]
    sounds : Vec<Sound>,
    etymology_text : Option<String>
}

#[derive(Serialize, Deserialize, Clone)]
struct Sense {
    #[serde(default)]
    glosses : Vec<String>,
    #[serde(default)]
    examples : Vec<Example>,
    #[serde(default)]
    tags : Vec<String>
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
    code : Option<String>,
    word : Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Sound {
    ipa : Option<String>,
    enpr: Option<String>,
    #[serde(default)]
    tags: Vec<String>
}

impl DictionaryEntry {
    pub fn to_pretty_string(&self) -> String {
    let senses : String = self.senses
        .clone()
        .into_iter()
        .enumerate()
        .fold(String::new(), |res, (i, sense)| {
                    res + &formatdoc!("
                                {}. {} {}
                                {}
                                ",
                                i.to_string().bold(), format_tags(&sense.tags).bold(),
                                format_glosses(&sense.glosses),
                                format_examples(&sense.examples))
        });
    /*
    if self.etymology_text.is_some(){
        println!("{}\n", self.etymology_text.clone().unwrap());
    }
    */

    return formatdoc!("
              -------------------------------------------
              {} ({})
              -------------------------------------------
              {}
              -------------------------------------------
              {}
              -------------------------------------------
              {}
              -------------------------------------------
              ", &self.word.clone().green().bold(),
                 self.pos, format_sounds(&self.sounds),
                 senses, format_translations(&self.translations));
    }
}

fn format_sounds(sounds : &Vec<Sound>) -> ColoredString {
    match sounds_to_strings(sounds)
        .as_slice() {
        [] => "Pronunciation:".bold(),
        xs => {
           return xs.into_iter()
               .enumerate()
               .fold("Pronunciation:\n".bold(), |res, (i, sound)| {
                    return formatdoc!("{} {}. {}\n",
                                      res, i.to_string().italic(), sound).normal();
               })
        }
    }
}

fn sounds_to_strings(sounds : &Vec<Sound>) -> Vec<String> {
    let mut results : Vec<String> = Vec::new();
    for sound in sounds {
        sound.ipa
            .clone()
            .map(|s| results.push(format!("IPA:  {} {}", s, format_tags(&sound.tags))));
        sound.enpr
            .clone()
            .map(|s| results.push(format!("enPr: {} {}", s, format_tags(&sound.tags))));
    }
    return results;
}

fn format_tags (tags : &Vec<String>) -> String {
    match tags.as_slice() {
        [] => return String::new(),
        xs => return format!("({})", xs.join(", "))
    }
}

fn format_glosses(glosses : &Vec<String>) -> String{
    match glosses.as_slice() {
        [gloss] => return gloss.to_string(),
        _       => return String::new()
    }
}

fn format_examples(examples : &Vec<Example>) -> String{
    match examples.as_slice() {
        [] => String::new(),
        xs => {
           return xs.into_iter()
               .enumerate()
               .fold("\n".to_string(), |res, (i, example)| {
                    return res + &formatdoc!(" {}. {}\n",
                                             i.to_string().italic(),
                                             example.text);
                })
        }
    }
}

fn format_translations(translations : &Vec<Translation>) -> ColoredString {
    match translations.as_slice() {
        [] => "Translations:".bold(),
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
               .fold("Translations:\n".bold(), |res, translation| {
                    return format!("{} {}) {}\n",
                         res,
                         translation.lang.italic(),
                         translation.word.clone().unwrap_or_else(String::new)).normal();
                })
        }
    }
}

pub fn parse(line : &String) -> Result<DictionaryEntry> {
    return serde_json::from_str(line).map_err(|err| anyhow::Error::new(err));
}
