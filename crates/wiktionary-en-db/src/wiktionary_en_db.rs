use anyhow::{bail, Context, Result};
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use utilities::language::*;

use polodb_core::bson::doc;
use polodb_core::{Collection, CollectionT, Database, IndexModel};
use wiktionary_entities::wiktionary_entity::*;

use std::fs::File;

pub fn get_polo_db_path() -> PathBuf {
    return PathBuf::from(utilities::DICTIONARY_POLO_DB_DIR!());
}

pub fn get_db_path(path: Option<String>, language: &Option<Language>) -> PathBuf {
    if let Some(path) = path {
        return PathBuf::from(path);
    }
    return PathBuf::from(utilities::DICTIONARY_DB_PATH!(language
        .unwrap_or_default()
        .value()));
}

fn find_by_word_in_collection(
    term: &String,
    collection: Collection<DictionaryEntry>,
) -> Result<()> {
    let result = collection.find(doc! { "word" : term}).run();
    match result {
        Ok(entries) => {
            for entry in entries {
                println!("{}", entry?.to_pretty_string());
            }
            return Ok(());
        }
        Err(err) => bail!(err),
    }
}

pub fn find_by_word(term: &String, language: &Language, create_indices: bool) -> Result<()> {
    let db_result = Database::open_path(get_polo_db_path());
    match db_result {
        Ok(db) => {
            let collection = db.collection::<DictionaryEntry>(&language.value());
            if create_indices {
                create_index_on_word(&collection)?;
            }
            find_by_word_in_collection(term, collection)?;
            return Ok(());
        }
        _ => bail!("No such DB file"),
    }
}

fn insert_wiktionary_file_into_db(
    db: Database,
    file_reader: BufReader<File>,
    language: &Language,
) -> Result<()> {
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
            }
            _ => bail!(format!("couldn't read line {}", i)),
        }
        count = count + 1;
    }
    println!("iterated over {} entries", count);
    return Ok(());
}

pub fn insert_wiktionary_file(file_reader: BufReader<File>, language: &Language) -> Result<()> {
    let db_result = Database::open_path(get_polo_db_path());

    match db_result {
        Ok(db) => return insert_wiktionary_file_into_db(db, file_reader, language),
        Err(err) => bail!(err),
    }
}

fn parse_line(line: &String, i: usize) -> Result<DictionaryEntry> {
    parse_entry(line).with_context(|| format!("Couldn't parse line {} in DB file.", i))
}

pub fn create_index_on_word(collection: &Collection<DictionaryEntry>) -> Result<()> {
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
