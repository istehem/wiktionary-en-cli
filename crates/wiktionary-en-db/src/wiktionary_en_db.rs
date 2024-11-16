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
) -> Result<Vec<DictionaryEntry>> {
    let mut result = Vec::new();
    let search_result = collection.find(doc! { "word" : term}).run();
    match search_result {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    result.push(entry);
                }
            }
            return Ok(result);
        }
        Err(err) => bail!(err),
    }
}

pub fn find_by_word(term: &String, language: &Language) -> Result<Vec<DictionaryEntry>> {
    let db_result = Database::open_path(get_polo_db_path());
    match db_result {
        Ok(db) => {
            let collection = db.collection::<DictionaryEntry>(&language.value());
            let result = find_by_word_in_collection(term, collection)?;
            return Ok(result);
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
    let collection = db.collection::<DictionaryEntry>(language.value().as_str());
    create_index_on_word(&collection)?;
    let mut all_entries = Vec::new();
    for (i, line) in file_reader.lines().enumerate() {
        match line {
            Ok(ok_line) => {
                let dictionary_entry = parse_line(&ok_line, i)?;
                all_entries.push(dictionary_entry);
            }
            _ => bail!(format!("couldn't read line {}", i)),
        }
        count = count + 1;
    }
    let batch_insert = collection.insert_many(all_entries);
    if let Err(err) = batch_insert {
        bail!(err);
    }

    println!(
        "inserted {} entries for language {}",
        count,
        &language.value()
    );
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
