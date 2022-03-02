use clap::Parser;
use std::io::{prelude::*, BufReader};
use std::fs::File;
use anyhow::{Result};
use serde::{Deserialize, Serialize};
use rand::thread_rng;
use rand::Rng;
use std::path::Path;
use indoc::{printdoc, formatdoc};
use colored::Colorize;
use std::env;
use self::Language::*;
use edit_distance::edit_distance;

static DEFAULT_DB_SUB_PATH: &str = "files/wiktionary-en.json";

/// An English Dictionary
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Override dictionary db file to use
    #[clap(long)]
    db_path: Option<String>,
    /// A word to search for; omitting it will yield a random entry
    search_term: Option<String>,
    /// Maximal number of results
    #[clap(short, long, default_value = "1")]
    max_results : usize
}

#[derive(Serialize, Deserialize, Clone)]
struct Data {
    lang_code : String,
    #[serde(default)]
    word : String,
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
struct Translation {
    lang : String,
    #[serde(default)]
    code : Option<String>,
    #[serde(default)]
    word : Option<String>,
}

#[derive(Copy, Clone)]
enum Language {
    EN,
    DE,
    FR,
    ES,
    SV
}

impl Language {
    fn value(&self) -> String {
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


fn get_file_reader(path: &Path) -> BufReader<File> {
    BufReader::new(File::open(path).unwrap())
}

fn format_glosses(glosses : &Vec<String>) -> String{
    match glosses.as_slice() {
        [] => "Glossaries".to_string(),
        xs => {
           return xs.into_iter()
               .enumerate()
               .fold("Glossaries:\n".to_string(), | res, (i, gloss) | {
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
               .fold("Examples:\n".to_string(), | res, (i, example) | {
                    return res + &formatdoc!(" {}) {}\n", i, example.text);
                })
        }
    }
}

fn empty_string() -> Option<String> {
    return Some(String::new());
}


fn format_translations(translations : &Vec<Translation>) -> String {
    match translations.as_slice() {
        [] => "Translations:".to_string(),
        _  => {
            let langs : Vec<Option<String>> = Language::iterator()
                .map(| lang | { Some(lang.value()) })
                .collect();
            let mut filtered_translations : Vec<Translation> = translations.clone()
                .into_iter()
                .filter(| translation | { langs.contains(&&translation.code) })
                .collect();
            filtered_translations.sort_by(| t1, t2 | t1.lang.cmp(&t2.lang));
            return filtered_translations.into_iter()
               .fold("Translations:\n".to_string(), | res, translation | {
                    return res + &formatdoc!(" {}) {}\n",
                         translation.lang,
                         translation.word.clone().or_else(empty_string).unwrap());
                })
        }
    }
}

fn print_entry(json : &Data) {
    let senses : String = json.senses
        .clone()
        .into_iter()
        .enumerate()
        .fold(String::new(), | res, (_i, sense) | {
                    res + &formatdoc!("
                                {}
                                {}
                                -------------------------------------------
                                ",
                                format_glosses(&sense.glosses),
                                format_examples(&sense.examples))
        });

    printdoc!("
              -------------------------------------------
              {} ({})
              -------------------------------------------
              {}
              {}
              -------------------------------------------
              ", &json.word.clone().green(),
                 json.pos, senses, format_translations(&json.translations));
}

fn random_entry(input_path : &Path) -> Result<()> {
    let lines = get_file_reader(input_path).lines();
    let n_entries : usize = get_file_reader(input_path).lines().count();
    let mut rng = thread_rng();
    let random_entry_number: usize = rng.gen_range(0, n_entries - 1);
    
    for (i, line) in lines.enumerate() {
        if i == random_entry_number {
            let json : Data = serde_json::from_str(&line.unwrap()).unwrap();
            print_entry(&json);
        }
    }
    return Ok(());
}

fn search_entry(input_path : &Path, term : String, max_results : usize) -> Result<()> {
    let lines = get_file_reader(input_path).lines();
    let mut result : Option<Data> = None;
    let mut full_matches : Vec<Data> = Vec::new();
    let mut min_distance = usize::MAX;
    for line in lines {
        let json : Data = serde_json::from_str(&line.unwrap()).unwrap();
        let distance = edit_distance(&json.word, &term);
        if distance < min_distance {
            min_distance = distance;
            result = Some(json.clone());
        }
        if distance == 0 {
            full_matches.push(json);
        }
        if full_matches.len() == max_results {
            break;
        }
    }
    match result {
        None        => println!("{}", "No results"),
        Some(json)  => {
            if min_distance != 0 {
                printdoc!("
                          ###########################################
                          No result for {}.
                          Did you mean  {}?
                          ###########################################
                          ",
                          &term.red(), &json.word.yellow());
            }
            for res in full_matches {
                print_entry(&res);
            }
        }
    };
    return Ok(());
}


fn run(term : Option<String>, max_results : usize, path : &Path) -> Result<()> {
    match term {
       Some(s) => return search_entry(&path, s, max_results),
       None    => return random_entry(&path)
    };
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut default = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    default.push(DEFAULT_DB_SUB_PATH);
    match args.db_path {
       Some(path) => return run(args.search_term, args.max_results, Path::new(&path)),
       None       => return run(args.search_term, args.max_results, default.as_path())
    };
}
