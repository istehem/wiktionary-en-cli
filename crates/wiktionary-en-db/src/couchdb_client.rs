use anyhow::{bail, Context, Result};
use couch_rs::database::Database;
use couch_rs::document::DocumentCollection;
use couch_rs::types::find::FindQuery;
use couch_rs::types::find::SortSpec;
use couch_rs::types::index::IndexFields;
use couch_rs::Client;
use serde_json::json;
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::vec::Vec;
use tokio::sync::Mutex;
use utilities::language::Language;
use wiktionary_en_entities::dictionary_entry::DictionaryEntry;

macro_rules! extension_database {
    ($language:expr, $extension_name:expr) => {
        format!("extension_{}_{}", $language, $extension_name).as_str()
    };
}

#[derive(Clone)]
pub struct DbClient {
    client: Client,
    database: Database,
    language: Language,
}

#[derive(Clone)]
pub struct DbClientMutex {
    pub client: Arc<Mutex<DbClient>>,
}

impl DbClientMutex {
    pub fn from(db_client: DbClient) -> Self {
        let client = Arc::new(Mutex::new(db_client));
        Self { client }
    }

    pub async fn init(language: Language) -> Result<Self> {
        Ok(Self::from(DbClient::init(language).await?))
    }
}

pub struct Document {
    pub document: Value,
}

impl DbClient {
    pub async fn init(language: Language) -> Result<Self> {
        let client = couch_rs::Client::new(
            &env::var("COUCH_DB_HOST").unwrap_or(env!("COUCH_DB_HOST").to_string()),
            env!("COUCH_DB_USER"),
            env!("COUCH_DB_PASSWORD"),
        )?;
        let database = client.db(language.to_string().as_str()).await?;
        Ok(Self {
            client,
            database,
            language,
        })
    }

    pub async fn find_by_word(&self, term: &str) -> Result<Vec<DictionaryEntry>> {
        let query = FindQuery::new(
            json!({ "word": term }), // Replace "status" and "active" with your field and term
        );

        let docs: DocumentCollection<DictionaryEntry> = self.database.find(&query).await?;
        Ok(docs.rows)
    }

    async fn find_raw_in_extension_collection(
        &self,
        extension_name: &str,
        query: Document,
    ) -> Result<Vec<Value>> {
        let extension_db = self
            .client
            .db(extension_database!(self.language, extension_name))
            .await?;

        let mut result: Vec<Value> = Vec::new();
        let mut bookmark: Option<String> = None;
        let mut find_query = FindQuery::new(query.document);

        loop {
            find_query.bookmark = bookmark;
            let query_result = extension_db.find_raw(&find_query).await?;
            let page: Vec<Value> = query_result.rows;
            if page.is_empty() {
                break;
            }
            result.extend(page);
            bookmark = query_result.bookmark;
        }
        Ok(result)
    }

    pub async fn find_in_extension_collection(
        &self,
        extension_name: &str,
        query: Document,
    ) -> Result<Vec<Document>> {
        Ok(self
            .find_raw_in_extension_collection(extension_name, query)
            .await?
            .into_iter()
            .map(Document::from)
            .collect())
    }

    pub async fn find_one_in_extension_collection(
        &self,
        extension_name: &str,
        document: Document,
    ) -> Result<Option<Document>> {
        let extension_db = self
            .client
            .db(extension_database!(self.language, extension_name))
            .await?;
        let result = extension_db
            .find_raw(&FindQuery::new(document.document).limit(1))
            .await?;

        Ok(result.rows.into_iter().next().map(Document::from))
    }

    pub async fn insert_one_into_extension_collection(
        &self,
        extension_name: &str,
        mut document: Document,
    ) -> Result<Document> {
        let extension_db = self
            .client
            .db(extension_database!(self.language, extension_name))
            .await?;
        let result = extension_db.create(&mut document.document).await?;
        Ok(Document::from(json!({"_id": result.id})))
    }

    pub async fn update_one_in_extension_collection(
        &self,
        extension_name: &str,
        mut document: Document,
    ) -> Result<Document> {
        let extension_db = self
            .client
            .db(extension_database!(self.language, extension_name))
            .await?;
        let result = extension_db.save(&mut document.document).await?;
        Ok(Document::from(json!({"_rev": result.rev})))
    }

    // TODO this will query all documents and leads to bad performance
    pub async fn delete_many_in_extension_collection(
        &self,
        extension_name: &str,
        query: Document,
    ) -> Result<usize> {
        let mut documents = self
            .find_raw_in_extension_collection(extension_name, query)
            .await?;
        let extension_db = self
            .client
            .db(extension_database!(self.language, extension_name))
            .await?;
        for document in &mut documents {
            document["_deleted"] = json!(true);
        }
        extension_db.bulk_docs(&mut documents).await?;
        Ok(documents.len())
    }

    // TODO this returns more than the actual number of pure extension entries
    pub async fn count_documents_in_extension_collection(
        &self,
        extension_name: &str,
    ) -> Result<u64> {
        let info = self
            .client
            .get_info(extension_database!(self.language, extension_name))
            .await?;

        Ok(info.doc_count)
    }

    pub async fn create_index_for_extension_collection(
        &self,
        extension_name: &str,
        keys: Document,
    ) -> Result<()> {
        let extension_db = self
            .client
            .db(extension_database!(self.language, extension_name))
            .await?;
        let mut fields = Vec::new();

        if let Some(key_values) = keys.document.as_object() {
            for (key, _) in key_values {
                fields.push(SortSpec::Simple(key.to_string()));
            }
        } else {
            bail!("no index keys supplied.")
        }

        let index_def = IndexFields { fields };
        let result = extension_db
            .insert_index("extension-index", index_def, None, None)
            .await?;
        if let Some(error) = result.error {
            bail!(error);
        }
        Ok(())
    }

    // TODO this returns more than the actual number of word entries
    pub async fn number_of_entries(&self) -> Result<u64> {
        let info = self
            .client
            .get_info(self.language.to_string().as_str())
            .await?;

        Ok(info.doc_count)
    }

    pub async fn insert_wiktionary_file(
        &mut self,
        file_reader: BufReader<File>,
        force: bool,
    ) -> Result<usize> {
        let count = self.number_of_entries().await?;
        if !force && count > 0 {
            bail!(
                "dictionary already contains {} entries for language {}, use force to override",
                count,
                self.language
            );
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

        if count > 0 {
            self.recreate_database().await?;
        }
        self.create_index_on_word().await?;

        let mut total_count = 0;

        let batch_size = 5000;
        for chunk in all_entries.chunks_mut(batch_size) {
            let result = self.database.bulk_docs(chunk).await?;
            total_count += result.len();
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

    async fn recreate_database(&mut self) -> Result<()> {
        let database_name = self.database.name();
        self.client.destroy_db(self.database.name()).await?;
        self.database = self.client.db(database_name).await?;
        Ok(())
    }
}

fn parse_line(line: &str, i: usize) -> Result<DictionaryEntry> {
    line.parse()
        .with_context(|| format!("Couldn't parse line {} in DB file.", i))
}

impl Document {
    pub fn from(value: Value) -> Self {
        Self { document: value }
    }
}
