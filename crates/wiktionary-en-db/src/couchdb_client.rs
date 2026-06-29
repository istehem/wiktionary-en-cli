use anyhow::{bail, Ok, Result};
use bson::document::Document;
use bson::Bson;
use couch_rs::Client;
use std::sync::{Arc, Mutex};
use std::vec::Vec;
use utilities::language::Language;
use wiktionary_en_entities::dictionary_entry::DictionaryEntry;

#[derive(Clone)]
pub struct DbClient {
    couch_client: Client,
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
    pub fn find_by_word(&self, _term: &str) -> Result<Vec<DictionaryEntry>> {
        Ok(Vec::new())
    }
    pub fn find_in_extension_collection(
        &self,
        _extension_name: &str,
        _document: ExtensionDocument,
    ) -> Result<Vec<ExtensionDocument>> {
        Ok(Vec::new())
    }
    pub fn find_one_in_extension_collection(
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
}

impl ExtensionDocument {
    pub fn from(document: Document) -> Self {
        Self { document }
    }
}
