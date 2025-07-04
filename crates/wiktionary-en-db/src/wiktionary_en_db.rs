use anyhow::{bail, Context, Result};
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use utilities::language::*;

use polodb_core::bson::doc;
use polodb_core::{Collection, CollectionT, Database, IndexModel};
use rand::{rng, Rng};
use wiktionary_en_entities::wiktionary_entry::*;

use std::fs::File;

pub struct WiktionaryDbClient {
    pub database: Database,
    pub language: Language,
}

impl WiktionaryDbClient {
    pub fn init(language: Language) -> Result<Self> {
        let database = Database::open_path(get_polo_db_path())?;
        Ok(Self { database, language })
    }

    fn collection(&self) -> Collection<DictionaryEntry> {
        self.database
            .collection::<DictionaryEntry>(&self.language.value())
    }

    pub fn find_by_word(&self, term: &str) -> Result<Vec<DictionaryEntry>> {
        find_by_word_in_collection(term, &self.collection())
    }

    pub fn insert_wiktionary_file(&self, file_reader: BufReader<File>, force: bool) -> Result<()> {
        insert_wiktionary_file_into_collection(
            &self.collection(),
            file_reader,
            &self.language,
            force,
        )
    }
}

pub fn get_polo_db_path() -> PathBuf {
    PathBuf::from(utilities::DICTIONARY_POLO_DB_DIR!())
}

pub fn get_db_path(path: Option<String>, language: &Option<Language>) -> PathBuf {
    if let Some(path) = path {
        return PathBuf::from(path);
    }
    PathBuf::from(utilities::DICTIONARY_DB_PATH!(language
        .unwrap_or_default()
        .value()))
}

fn delete_all_in_collection(collection: &Collection<DictionaryEntry>) -> Result<u64> {
    let delete_result = collection.delete_many(doc! {});
    match delete_result {
        Ok(delete_result) => Ok(delete_result.deleted_count),
        Err(err) => bail!(err),
    }
}

fn find_by_word_in_collection(
    term: &str,
    collection: &Collection<DictionaryEntry>,
) -> Result<Vec<DictionaryEntry>> {
    let mut result = Vec::new();
    let search_result = collection.find(doc! { "word" : term}).run();
    match search_result {
        Ok(entries) => {
            for entry in entries {
                result.push(entry?);
            }
            Ok(result)
        }
        Err(err) => bail!(err),
    }
}

/// This is very inefficient.
/// In MongoDB we could use the $sample aggregate, however this is lacking in PoloDB.
/// db.collectionName.aggregate([{$sample: {size: 1}}]);
fn random_entry_in_collection(collection: &Collection<DictionaryEntry>) -> Result<DictionaryEntry> {
    let n_entries = number_of_entries_in_collection(collection)?;
    let random_entry_number = rng().random_range(0..n_entries - 1);
    let result = collection
        .find(doc! {})
        .skip(random_entry_number)
        .limit(1)
        .run();
    match result {
        Ok(mut cursor) => {
            if let Some(entry) = cursor.next() {
                return Ok(entry?);
            }
            bail!("no entries found")
        }
        Err(err) => bail!(err),
    }
}

pub fn random_entry_for_language(language: &Language) -> Result<DictionaryEntry> {
    let db_result = Database::open_path(get_polo_db_path());
    match db_result {
        Ok(db) => {
            let collection = db.collection::<DictionaryEntry>(&language.value());
            Ok(random_entry_in_collection(&collection)?)
        }
        Err(err) => bail!(err),
    }
}

pub fn number_of_entries_for_language(language: &Language) -> Result<u64> {
    let db_result = Database::open_path(get_polo_db_path());
    match db_result {
        Ok(db) => {
            let collection = db.collection::<DictionaryEntry>(&language.value());
            number_of_entries_in_collection(&collection)
        }
        Err(err) => bail!(err),
    }
}

fn number_of_entries_in_collection(collection: &Collection<DictionaryEntry>) -> Result<u64> {
    let count: Result<u64, polodb_core::Error> = collection.count_documents();
    match count {
        Ok(count) => Ok(count),
        Err(err) => bail!(err),
    }
}

fn insert_wiktionary_file_into_collection(
    collection: &Collection<DictionaryEntry>,
    file_reader: BufReader<File>,
    language: &Language,
    force: bool,
) -> Result<()> {
    if !force {
        let count = number_of_entries_in_collection(collection)?;
        if count > 0 {
            bail!(
                "dictionary already contains {} entries for language {}, use force to override",
                count,
                language.value()
            );
        }
    }

    delete_all_in_collection(collection)?;
    create_index_on_word(collection)?;

    let mut count = 0;
    let mut all_entries = Vec::new();
    for (i, line) in file_reader.lines().enumerate() {
        match line {
            Ok(ok_line) => {
                let dictionary_entry = parse_line(&ok_line, i)?;
                all_entries.push(dictionary_entry);
            }
            _ => bail!("couldn't read line {}", i),
        }
        count += 1;
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
    Ok(())
}

fn parse_line(line: &str, i: usize) -> Result<DictionaryEntry> {
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

    Ok(())
}
