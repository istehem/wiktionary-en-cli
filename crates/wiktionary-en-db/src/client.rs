use anyhow::{bail, Context, Result};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use utilities::language::Language;

use bson::document::Document;
use bson::Bson;
use polodb_core::bson::doc;
use polodb_core::{Collection, CollectionT, Database, IndexModel};
use rand::{rng, Rng};
use wiktionary_en_entities::dictionary_entry::DictionaryEntry;
use wiktionary_en_entities::{history_collection, history_entry::HistoryEntry};

use std::fs::File;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct DbClient {
    database: Database,
    language: Language,
}

#[derive(Clone)]
pub struct DbClientMutex {
    pub client: Arc<Mutex<DbClient>>,
}

macro_rules! extension_collection {
    ($language:expr, $extension_name:expr) => {
        format!("extension_{}_{}", $language, $extension_name)
    };
}

impl DbClientMutex {
    pub fn from(db_client: DbClient) -> Self {
        let client = Arc::new(Mutex::new(db_client));
        Self { client }
    }

    pub fn init(language: Language) -> Result<Self> {
        Ok(Self::from(DbClient::init(language)?))
    }
}

pub struct ExtensionDocument {
    pub document: Document,
}

impl ExtensionDocument {
    pub fn from(document: Document) -> Self {
        Self { document }
    }
}

impl DbClient {
    pub fn init(language: Language) -> Result<Self> {
        let database = Database::open_path(get_polo_db_path())?;
        Ok(Self { database, language })
    }

    fn collection(&self) -> Collection<DictionaryEntry> {
        self.database
            .collection::<DictionaryEntry>(&self.language.to_string())
    }

    pub fn history_collection(&self) -> Collection<HistoryEntry> {
        self.database
            .collection::<HistoryEntry>(&history_collection!(&self.language))
    }

    pub fn extension_collection(&self, extension_name: &str) -> Collection<Document> {
        self.database
            .collection::<Document>(&extension_collection!(&self.language, extension_name))
    }

    pub fn history_docs_collection(&self) -> Collection<Document> {
        self.database
            .collection::<Document>(&history_collection!(&self.language))
    }

    pub fn find_by_word(&self, term: &str) -> Result<Vec<DictionaryEntry>> {
        find_by_word_in_collection(term, &self.collection())
    }

    pub fn upsert_into_history(&self, term: &str) -> Result<()> {
        let collection = self.history_collection();
        if let Some(mut entry) = self.find_in_history_by_word(term)? {
            entry.tick();
            collection.update_one(
                doc! {
                    "word": &entry.word,
                },
                doc! {
                   "$set": doc!{
                     "last_seen_at": entry.last_seen_at.timestamp(),
                     "now_seen_at": entry.now_seen_at.timestamp(),
                     "count": entry.count as u32,
                }},
            )?;
        } else {
            collection.insert_one(HistoryEntry::from(term.to_string()))?;
        }
        Ok(())
    }

    pub fn insert_one_into_extension_collection(
        &self,
        extension_name: &str,
        document: ExtensionDocument,
    ) -> Result<Bson> {
        let collection = self.extension_collection(extension_name);
        let result = collection.insert_one(document.document)?;
        Ok(result.inserted_id)
    }

    pub fn update_one_in_extension_collection(
        &self,
        extension_name: &str,
        query: ExtensionDocument,
        update: ExtensionDocument,
    ) -> Result<u64> {
        let collection = self.extension_collection(extension_name);
        let result = collection.update_one(
            query.document,
            doc! {
                "$set" : update.document
            },
        )?;
        Ok(result.modified_count)
    }

    pub fn find_in_extension_collection(
        &self,
        extension_name: &str,
        document: ExtensionDocument,
    ) -> Result<Vec<ExtensionDocument>> {
        let collection = self.extension_collection(extension_name);
        let search_result = collection.find(document.document).run()?;
        let mut result: Vec<ExtensionDocument> = Vec::new();
        for document in search_result {
            result.push(ExtensionDocument::from(document?));
        }
        Ok(result)
    }

    pub fn find_one_in_extension_collection(
        &self,
        extension_name: &str,
        document: ExtensionDocument,
    ) -> Result<Option<ExtensionDocument>> {
        let collection = self.extension_collection(extension_name);
        let result = collection.find_one(document.document)?;
        if let Some(document) = result {
            return Ok(Some(ExtensionDocument::from(document)));
        }
        Ok(None)
    }

    fn find_in_history_by_word(&self, term: &str) -> Result<Option<HistoryEntry>> {
        let collection = self.history_collection();
        let result = collection.find_one(doc! { "word" : term})?;
        Ok(result)
    }

    pub fn delete_history(&self) -> Result<u64> {
        let collection = self.history_collection();
        delete_all_in_collection(&collection)
    }

    pub fn insert_wiktionary_file(
        &self,
        file_reader: BufReader<File>,
        force: bool,
    ) -> Result<usize> {
        create_history_index(&self.history_collection())?;
        insert_wiktionary_file_into_collection(
            &self.collection(),
            file_reader,
            &self.language,
            force,
        )
    }

    pub fn number_of_entries(&self) -> Result<u64> {
        number_of_entries_in_collection(&self.collection())
    }

    pub fn random_entry(&self) -> Result<DictionaryEntry> {
        random_entry_in_collection(&self.collection())
    }
}

fn create_history_index(collection: &Collection<HistoryEntry>) -> Result<()> {
    collection.create_index(IndexModel {
        keys: doc! {
            "word": 1,
        },
        options: None,
    })?;
    Ok(())
}

fn get_polo_db_path() -> PathBuf {
    PathBuf::from(utilities::DICTIONARY_POLO_DB_DIR!())
}

fn delete_all_in_collection<T>(collection: &Collection<T>) -> Result<u64> {
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
    let search_result = collection.find(doc! { "word" : term}).run()?;

    for entry in search_result {
        result.push(entry?);
    }

    Ok(result)
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
) -> Result<usize> {
    if !force {
        let count = number_of_entries_in_collection(collection)?;
        if count > 0 {
            bail!(
                "dictionary already contains {} entries for language {}, use force to override",
                count,
                language.to_string()
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

    Ok(count)
}

fn parse_line(line: &str, i: usize) -> Result<DictionaryEntry> {
    line.parse()
        .with_context(|| format!("Couldn't parse line {} in DB file.", i))
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
