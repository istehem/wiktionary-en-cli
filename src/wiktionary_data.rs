use serde::{Deserialize, Serialize};
use indoc::{formatdoc};
use colored::Colorize;
use anyhow::Result;
use colored::ColoredString;

pub mod language;

trait Join {
    fn join(&self, list : Vec<Self>) -> Self where Self: Sized;
}

impl Join for ColoredString {
    fn join(&self, list : Vec<ColoredString>) -> ColoredString {
        let mut res : ColoredString = "".normal();
        let len : usize = list.len();
        for (i, string) in list.iter().enumerate() {
            res = format!("{}{}", res, string).normal();
            if i < len - 1 {
                res = format!("{}{}", res, self).normal();
            }
        }
        return res.clone();
    }
}

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
              ", &self.word.clone().green().bold(), self.pos,
                 format_sounds(&self.sounds),
                 format_senses(&self.senses),
                 format_translations(&self.translations));
    }
}

fn senses_to_strings(senses : &Vec<Sense>) -> Vec<ColoredString> {
    return senses
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, sense)| format_sense(&sense, i))
        .collect();
}

fn format_senses(senses : &Vec<Sense>) -> ColoredString {
    return "\n\n".normal().join(senses_to_strings(senses));
}

fn format_sense(sense : &Sense, index : usize) -> ColoredString {
    let mut res : Vec<ColoredString> = Vec::new();
    let fst =  format!("{}. {} {}",
                            index.to_string().bold(),
                            format_tags(&sense.tags).bold(),
                            format_glosses(&sense.glosses)).normal();
    res.push(fst);
    if !sense.examples.is_empty() {
        res.push("\n".normal().join(examples_to_strings(&sense.examples)));
    }
    return "\n".normal().join(res);
}

fn format_sounds(sounds : &Vec<Sound>) -> ColoredString {
    let mut res : Vec<ColoredString> = Vec::new();
    res.push("Pronunciation".bold());
    if !sounds.is_empty() {
        res.push("\n".normal().join(sounds_to_strings(sounds)));
    }
    return "\n".normal().join(res);
}

fn sounds_to_strings(sounds : &Vec<Sound>) -> Vec<ColoredString> {
    let mut results : Vec<ColoredString> = Vec::new();
    for (i, sound) in sounds.into_iter().enumerate() {
        sound.ipa
            .clone()
            .map(|s| results.push(format!(" {}. IPA:  {} {}",
                                    i.to_string().italic(), s,
                                    format_tags(&sound.tags)).normal()));
        sound.enpr
            .clone()
            .map(|s| results.push(format!(" {}. enPr: {} {}",
                                     i.to_string().italic(), s,
                                     format_tags(&sound.tags)).normal()));
    }
    return results;
}

fn format_tags(tags : &Vec<String>) -> String {
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

fn examples_to_strings(examples : &Vec<Example>) -> Vec<ColoredString>{
    return examples.into_iter()
              .enumerate()
              .map(|(i, example)| format!(" {}. {}",
                                            i.to_string().italic(),
                                            example.text).normal())
              .collect();
}

fn translations_to_strings(translations : &Vec<Translation>) -> Vec<ColoredString> {
    let langs : Vec<Option<String>> = language::Language::iterator()
                .map(|lang| { Some(lang.value()) })
                .collect();
    let mut filtered_translations : Vec<Translation> = translations
        .clone()
        .into_iter()
        .filter(|translation| { langs.contains(&&translation.code) })
        .collect();
    filtered_translations.sort_by(|t1, t2| t1.lang.cmp(&t2.lang));
    return filtered_translations
        .into_iter()
        .map(|translation| format!(" {}) {}",
                            translation.lang.italic(),
                            translation.word.clone().unwrap_or_else(String::new)).normal())
        .collect();
}

fn format_translations(translations : &Vec<Translation>) -> ColoredString {
    let mut res : Vec<ColoredString> = Vec::new();
    res.push("Translations".bold());
    if !translations.is_empty() {
        res.push("\n".normal().join(translations_to_strings(translations)));
    }
    return "\n".normal().join(res);
}

pub fn parse(line : &String) -> Result<DictionaryEntry> {
    return serde_json::from_str(line).map_err(|err| anyhow::Error::new(err));
}
