use colored::ColoredString;
use colored::Colorize;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use textwrap::{fill, indent};

use utilities::colored_string_utils::*;
use utilities::language::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DictionaryEntry {
    lang_code: String,
    #[serde(default)]
    pub word: String,
    senses: Vec<Sense>,
    pos: String,
    #[serde(default)]
    translations: Vec<Translation>,
    #[serde(default)]
    sounds: Vec<Sound>,
    etymology_text: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Sense {
    #[serde(default)]
    glosses: Vec<String>,
    #[serde(default)]
    examples: Vec<Example>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Example {
    #[serde(alias = "ref")]
    #[serde(default)]
    reference: Option<String>,
    text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Translation {
    lang: String,
    code: Option<String>,
    word: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Sound {
    ipa: Option<String>,
    enpr: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
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

        return formatdoc!(
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
        );
    }
}

fn format_etymology(etymology: &Option<String>) -> Option<ColoredString> {
    let mut res: Vec<ColoredString> = Vec::new();
    res.push("Etymology:".bold());
    if etymology.is_some() {
        res.push(etymology.clone().unwrap().normal());
    } else {
        return None;
    }
    return Some(NEWLINE.normal().joinwrap(res, LINE_WRAP_AT));
}

fn senses_to_strings(senses: &Vec<Sense>) -> Vec<ColoredString> {
    return senses
        .clone()
        .into_iter()
        .enumerate()
        .map(|(i, sense)| format_sense(&sense, i))
        .collect();
}

fn format_senses(senses: &Vec<Sense>) -> Option<ColoredString> {
    if senses.is_empty() {
        return None;
    }
    return Some(NEWLINE.normal().join(senses_to_strings(senses)));
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
    return NEWLINE.normal().join(res);
}

fn format_examples(examples: &Vec<Example>) -> ColoredString {
    return indent(
        &NEWLINE
            .normal()
            .joinwrap(examples_to_strings(&examples), LINE_WRAP_AT - 1),
        " ",
    )
    .normal();
}

fn format_sounds(sounds: &Vec<Sound>) -> Option<ColoredString> {
    let mut res: Vec<ColoredString> = Vec::new();
    res.push("Pronunciation".bold());
    if !sounds.is_empty() {
        res.push(NEWLINE.normal().join(sounds_to_strings(sounds)));
    } else {
        return None;
    }
    return Some(NEWLINE.normal().join(res));
}

fn sounds_to_strings(sounds: &Vec<Sound>) -> Vec<ColoredString> {
    let mut results: Vec<ColoredString> = Vec::new();
    for (i, sound) in sounds.into_iter().enumerate() {
        sound.ipa.clone().map(|s| {
            results.push(
                format!(
                    " {}. IPA:  {} {}",
                    i.to_string().italic(),
                    s,
                    format_tags(&sound.tags)
                )
                .normal(),
            )
        });
        sound.enpr.clone().map(|s| {
            results.push(
                format!(
                    " {}. enPr: {} {}",
                    i.to_string().italic(),
                    s,
                    format_tags(&sound.tags)
                )
                .normal(),
            )
        });
    }
    return results;
}

fn format_tags(tags: &Vec<String>) -> String {
    match tags.as_slice() {
        [] => return String::new(),
        xs => return format!("({})", xs.join(", ")),
    }
}

fn format_glosses(glosses: &Vec<String>) -> String {
    match glosses.as_slice() {
        [gloss] => return gloss.to_string(),
        _ => return String::new(),
    }
}

fn examples_to_strings(examples: &Vec<Example>) -> Vec<ColoredString> {
    return examples
        .into_iter()
        .enumerate()
        .map(|(i, example)| format!("{}. {}", i.to_string().italic(), example.text).normal())
        .collect();
}

fn translations_to_strings(translations: &Vec<Translation>) -> Vec<ColoredString> {
    let langs: Vec<Option<String>> = Language::iterator()
        .map(|lang| Some(lang.value()))
        .collect();
    let translations_as_set: HashSet<Translation> = translations
        .clone()
        .into_iter()
        .filter(|translation| langs.contains(&&translation.code))
        .collect();
    let mut filtered_translations = Vec::from_iter(translations_as_set);
    filtered_translations.sort_by(|t1, t2| t1.lang.cmp(&t2.lang));
    return filtered_translations
        .into_iter()
        .map(|translation| {
            format!(
                " {}) {}",
                translation.lang.italic(),
                translation.word.clone().unwrap_or_else(String::new)
            )
            .normal()
        })
        .collect();
}

fn format_translations(translations: &Vec<Translation>) -> Option<ColoredString> {
    let mut res: Vec<ColoredString> = Vec::new();
    res.push("Translations".bold());
    if !translations.is_empty() {
        res.push(NEWLINE.normal().join(translations_to_strings(translations)));
    } else {
        return None;
    }
    return Some(NEWLINE.normal().join(res));
}
