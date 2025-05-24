use anyhow::Result;
use colored::ColoredString;
use colored::Colorize;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use textwrap::{fill, indent};

use utilities::anyhow_serde;
use utilities::colored_string_utils::*;
use utilities::language::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictionaryEntry {
    pub lang_code: String,
    pub word: String,
    pub senses: Vec<Sense>,
    pub pos: String,
    #[serde(default)]
    pub translations: Vec<Translation>,
    #[serde(default)]
    pub sounds: Vec<Sound>,
    pub etymology_text: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sense {
    #[serde(default)]
    pub glosses: Vec<String>,
    #[serde(default)]
    pub examples: Vec<Example>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Example {
    #[serde(alias = "ref")]
    #[serde(default)]
    pub reference: Option<String>,
    pub text: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Translation {
    pub lang: String,
    pub code: Option<String>,
    pub word: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sound {
    pub ipa: Option<String>,
    pub enpr: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub fn parse_entry(entry_string: &str) -> Result<DictionaryEntry> {
    anyhow_serde::from_str(entry_string)
}

impl DictionaryEntry {
    pub fn to_pretty_string(&self) -> String {
        let mut entries: Vec<ColoredString> = Vec::new();
        let etymology = format_etymology(&self.etymology_text);
        let sounds = format_sounds(&self.sounds);
        let senses = format_senses(&self.senses);
        let translations = format_translations(&self.translations);

        if let Some(etymology) = etymology {
            entries.push(etymology);
        }
        if let Some(sounds) = sounds {
            entries.push(sounds);
        }
        if let Some(senses) = senses {
            entries.push(senses);
        }
        if let Some(translations) = translations {
            entries.push(translations);
        }
        let horizontal_line = horizontal_line();

        formatdoc!(
            "
            {}
            {} ({})
            {}
            {}
            ",
            horizontal_line,
            self.word.green().bold(),
            self.pos,
            horizontal_line,
            format!("{}{}{}", NEWLINE, horizontal_line, NEWLINE)
                .normal()
                .join(entries)
        )
    }
}

impl fmt::Display for DictionaryEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_pretty_string())
    }
}

fn format_etymology(etymology: &Option<String>) -> Option<ColoredString> {
    if let Some(etymology) = etymology {
        let mut res: Vec<ColoredString> = Vec::new();
        res.push("Etymology:".bold());
        res.push(etymology.normal());
        return Some(NEWLINE.normal().joinwrap(res, LINE_WRAP_AT));
    }
    None
}

fn senses_to_strings(senses: &[Sense]) -> Vec<ColoredString> {
    senses
        .iter()
        .enumerate()
        .map(|(i, sense)| format_sense(sense, i))
        .collect()
}

fn format_senses(senses: &[Sense]) -> Option<ColoredString> {
    if senses.is_empty() {
        return None;
    }
    Some(NEWLINE.normal().join(senses_to_strings(senses)))
}

fn format_sense(sense: &Sense, index: usize) -> ColoredString {
    let mut res: Vec<ColoredString> = Vec::new();
    let title = format!(
        "{}. {}",
        index.to_string().bold(),
        format_tags(&sense.tags).bold()
    )
    .normal();
    res.push(title);
    res.push(fill(&format_glosses(&sense.glosses), LINE_WRAP_AT).normal());
    if !sense.examples.is_empty() {
        res.push(format_examples(&sense.examples));
    }
    NEWLINE.normal().join(res)
}

fn format_examples(examples: &[Example]) -> ColoredString {
    indent(
        &NEWLINE
            .normal()
            .joinwrap(examples_to_strings(examples), LINE_WRAP_AT - 1),
        " ",
    )
    .normal()
}

fn format_sounds(sounds: &[Sound]) -> Option<ColoredString> {
    if sounds.is_empty() {
        return None;
    }
    let mut res: Vec<ColoredString> = Vec::new();
    res.push("Pronunciation".bold());
    res.push(NEWLINE.normal().join(sounds_to_strings(sounds)));
    Some(NEWLINE.normal().join(res))
}

fn sounds_to_strings(sounds: &[Sound]) -> Vec<ColoredString> {
    let mut results: Vec<ColoredString> = Vec::new();
    for (i, sound) in sounds.iter().enumerate() {
       if let Some(s) = sound.ipa.as_ref() {
            results.push(
                format!(
                    " {}. IPA:  {} {}",
                    i.to_string().italic(),
                    s,
                    format_tags(&sound.tags)
                )
                .normal(),
            )
       }
       if let Some(s) = sound.enpr.as_ref() {
            results.push(
                format!(
                    " {}. enPr: {} {}",
                    i.to_string().italic(),
                    s,
                    format_tags(&sound.tags)
                )
                .normal(),
            )
       }
    }
    results
}

fn format_tags(tags: &Vec<String>) -> String {
    match tags.as_slice() {
        [] => String::new(),
        xs => format!("({})", xs.join(", ")),
    }
}

fn format_glosses(glosses: &Vec<String>) -> String {
    match glosses.as_slice() {
        [gloss] => gloss.to_string(),
        _ => String::new(),
    }
}

fn examples_to_strings(examples: &[Example]) -> Vec<ColoredString> {
    examples
        .iter()
        .enumerate()
        .map(|(i, example)| {
            format!(
                "{}. {}",
                i.to_string().italic(),
                example.text.clone().unwrap_or(String::new())
            )
            .normal()
        })
        .collect()
}

fn translations_to_strings(translations: &[Translation]) -> Vec<ColoredString> {
    let langs: Vec<Option<String>> = Language::iterator()
        .map(|lang| Some(lang.value()))
        .collect();
    let translations_as_set: HashSet<&Translation> = translations
        .iter()
        .filter(|translation| langs.contains(&translation.code))
        .collect();
    let mut filtered_translations = Vec::from_iter(translations_as_set);
    filtered_translations.sort_by(|t1, t2| t1.lang.cmp(&t2.lang));
    filtered_translations
        .iter()
        .map(|translation| {
            format!(
                " {}) {}",
                translation.lang.italic(),
                translation.word.as_ref().unwrap_or(&String::new())
            )
            .normal()
        })
        .collect()
}

fn format_translations(translations: &[Translation]) -> Option<ColoredString> {
    if translations.is_empty() {
        return None;
    }
    let mut res: Vec<ColoredString> = Vec::new();
    res.push("Translations".bold());
    res.push(NEWLINE.normal().join(translations_to_strings(translations)));
    Some(NEWLINE.normal().join(res))
}
