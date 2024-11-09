use anyhow::{bail, Context, Result};
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use utilities::file_utils;
use utilities::language::*;

use wiktionary_entities::wiktionary_entity::*;

use std::fs::File;

use clap::Parser;
use polodb_core::{CollectionT, Database, Collection, IndexModel};

use polodb_core::bson::doc;

/// Import Dictionary Data into Sonic
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// A word to search for; omitting it will yield a random entry
    search_term: String,
    /// Override dictionary db file to use
    #[clap(long, short = 'd')]
    db_path: Option<String>,
    /// Language to import
    #[clap(long, short = 'l')]
    language: Option<String>,
    /// Force import even if data still exists in the bucket
    #[clap(long, short = 'f')]
    force: bool,
    /// Create indices 
    #[clap(long, short = 'i')]
    create_indices: bool,
}

fn parse_line(line: &String, i: usize) -> Result<DictionaryEntry> {
    parse_entry(line).with_context(|| format!("Couldn't parse line {} in DB file.", i))
}

fn loop_and_insert(db: Database, file_reader: BufReader<File>, language: &Language) -> Result<()> {
    let mut count = 0;
    for (i, line) in file_reader.lines().enumerate() {
        match line {
            Ok(ok_line) => {
                let dictionary_entry = parse_line(&ok_line, i)?;
                let collection = db.collection::<DictionaryEntry>(language.value().as_str());
                let insert = collection.insert_one(dictionary_entry);
                if let Err(err) = insert {
                   bail!(err); 
                }
            },
            _ => bail!(format!("couldn't read line {}", i))
            
        } 
        count = count + 1;
    }
    println!("iterated over {} entries", count);
    return Ok(());
}

fn create_index_on_word(collection: &Collection<DictionaryEntry>) -> Result<()> {
    let result = collection.create_index(IndexModel {
        keys: doc! {
            "word": 1,
        },
        options: None,
    });
    if let Err(err) = result {
        bail!(err);
    }

    return Ok(());
}

fn persist(file_reader: BufReader<File>, language: &Language) -> Result<()> {
    let db_result = Database::open_path(get_polo_db_path());

    match db_result {
        Ok(db) => return loop_and_insert(db, file_reader, language),
        Err(err) => bail!(err),
    }
}

pub fn do_import(path: &Path, language: &Language) -> Result<()> {
    match file_utils::get_file_reader(path) {
        Ok(path) => return persist(path, language),
        _ => bail!("No such DB file: '{}'", path.display()),
    }
}

fn get_polo_db_path() -> PathBuf {
    return PathBuf::from(utilities::DICTIONARY_POLO_DB_DIR!());
}

fn get_db_path(path: Option<String>, language: &Option<Language>) -> PathBuf {
    if let Some(path) = path {
        return PathBuf::from(path);
    }
    return PathBuf::from(utilities::DICTIONARY_DB_PATH!(language
        .unwrap_or_default()
        .value()));
}

fn get_language(language: &Option<String>) -> Language {
    if let Some(language) = language {
        return Language::from_string(&language).unwrap_or_default();
    }
    return Language::EN;
}

fn execute_query(term: &String, collection: Collection<DictionaryEntry>) -> Result<()> {
    let result = collection.find(doc! { "word" : term}).run();
    match result {
        Ok(entries) => {
            for entry in entries {
                println!("{}", entry?.to_pretty_string());
            }
            return Ok(());
        },
        Err(err)   => bail!(err)
    }
}

fn run_on_db(term: &String, language: &Language, create_indices: bool) -> Result<()> {
    let db_result = Database::open_path(get_polo_db_path());
    match db_result {
        Ok(db) => {
            let collection = db.collection::<DictionaryEntry>(&language.value());
            if create_indices {
                create_index_on_word(&collection)?;       
            }
            execute_query(term, collection)?;
            return Ok(());
        },
        _ => bail!("No such DB file"),
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let language = get_language(&args.language);
    let db_path: PathBuf = get_db_path(args.db_path, &Some(language));
    if args.force {
        return do_import(&db_path, &language);
    } else {
        run_on_db(&args.search_term, &get_language(&args.language), args.create_indices)?;
    }
    return Ok(());
}
