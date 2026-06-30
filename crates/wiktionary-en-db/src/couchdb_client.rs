use anyhow::{bail, Context, Result};
use bson::document::Document;
use bson::Bson;
use couch_rs::database::Database;
use couch_rs::document::DocumentCollection;
use couch_rs::types::find::FindQuery;
use couch_rs::types::find::SortSpec;
use couch_rs::types::index::IndexFields;
use serde_json::json;
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::vec::Vec;
use utilities::language::Language;
use wiktionary_en_entities::dictionary_entry::DictionaryEntry;

#[derive(Clone)]
pub struct DbClient {
    database: Database,
    language: Language,
}

#[derive(Clone)]
pub struct DbClientMutex {
    pub client: Arc<Mutex<DbClient>>,
}

pub struct ExtensionDocument {
    pub document: Document,
}

impl DbClient {
    const DB_HOST: &str = "http://localhost:5984";

    pub async fn init(language: Language) -> Result<Self> {
        let couch_client = couch_rs::Client::new(Self::DB_HOST, "admin", "password")?;
        let database = couch_client.db(language.to_string().as_str()).await?;
        Ok(Self { database, language })
    }

    pub async fn find_by_word(&self, term: &str) -> Result<Vec<DictionaryEntry>> {
        let query = FindQuery::new(
            json!({ "word": term }), // Replace "status" and "active" with your field and term
        );

        let docs: DocumentCollection<DictionaryEntry> = self.database.find(&query).await?;
        Ok(docs.rows)
    }

    pub fn find_in_extension_collection(
        &self,
        _extension_name: &str,
        _document: ExtensionDocument,
    ) -> Result<Vec<ExtensionDocument>> {
        Ok(Vec::new())
    }
    pub async fn find_one_in_extension_collection(
        &self,
        _extension_name: &str,
        __document: ExtensionDocument,
    ) -> Result<Option<ExtensionDocument>> {
        Ok(None)
    }

    pub fn insert_one_into_extension_collection(
        &self,
        _extension_name: &str,
        _document: ExtensionDocument,
    ) -> Result<Bson> {
        bail!("not implemented yet!")
    }

    pub fn update_one_in_extension_collection(
        &self,
        _extension_name: &str,
        _query: ExtensionDocument,
        _update: ExtensionDocument,
    ) -> Result<u64> {
        bail!("not implemented yet!")
    }

    pub fn delete_many_in_extension_collection(
        &self,
        _extension_name: &str,
        _query: ExtensionDocument,
    ) -> Result<u64> {
        Ok(0)
    }

    pub fn count_documents_in_extension_collection(&self, _extension_name: &str) -> Result<u64> {
        Ok(0)
    }

    pub fn create_index_for_extension_collection(
        &self,
        _extension_name: &str,
        _keys: ExtensionDocument,
    ) -> Result<()> {
        Ok(())
    }

    async fn number_of_entries_in_collection(&self) -> Result<u32> {
        let mut query = FindQuery::find_all();
        query.limit = Some(0);
        let result: DocumentCollection<Value> = self.database.find_raw(&query).await?;
        Ok(result.total_rows)
    }

    pub async fn insert_wiktionary_file(
        &self,
        file_reader: BufReader<File>,
        force: bool,
    ) -> Result<usize> {
        if !force {
            let count = self.number_of_entries_in_collection().await?;
            if count > 0 {
                bail!(
                    "dictionary already contains {} entries for language {}, use force to override",
                    count,
                    self.language
                );
            }
        }
        let mut all_entries = Vec::new();
        for (i, line) in file_reader.lines().enumerate() {
            match line {
                Ok(ok_line) => {
                    let dictionary_entry = parse_line(&ok_line, i)?;
                    all_entries.push(dictionary_entry);
                }
                _ => bail!("couldn't read line {}", i),
            }
        }
        let mut total_count = 0;

        let batch_size = 2000;
        for chunk in all_entries.chunks_mut(batch_size) {
            let result = self.database.bulk_docs(chunk).await?;
            total_count = total_count + result.len();
        }

        Ok(total_count)
    }
    pub async fn create_index_on_word(&self) -> Result<bool> {
        let index_def = IndexFields {
            fields: vec![SortSpec::Simple("word".to_string())],
        };
        let result = self
            .database
            .insert_index("word-index", index_def, None, None)
            .await?;
        Ok(result.result.is_some())
    }
}

fn parse_line(line: &str, i: usize) -> Result<DictionaryEntry> {
    line.parse()
        .with_context(|| format!("Couldn't parse line {} in DB file.", i))
}

impl ExtensionDocument {
    pub fn from(document: Document) -> Self {
        Self { document }
    }
}
