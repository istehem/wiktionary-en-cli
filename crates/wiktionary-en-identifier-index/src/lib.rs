use anyhow::{bail, Result};
use std::path::PathBuf;
use utilities::file_utils;
use utilities::language::*;

mod wiktionary_channel;
use crate::wiktionary_channel::*;

pub mod indexing_stream;
use crate::indexing_stream::*;

pub fn statistics(language: &Language) -> Result<()> {
    let ingest_channel = WiktionaryIngestChannel::init(language)?;
    ingest_channel.statistics()
}

pub fn suggest(language: &Language, search_term: &String) -> Result<Vec<String>> {
    let search_channel = WiktionarySearchChannel::init(language)?;
    search_channel.suggest(search_term)
}

pub fn query(language: &Language, search_term: &String) -> Result<Vec<String>> {
    let search_channel = WiktionarySearchChannel::init(language)?;
    search_channel.query(search_term)
}

pub fn did_you_mean(language: &Language, search_term: &String) -> Result<Option<String>> {
    let search_channel = WiktionarySearchChannel::init(language)?;
    search_channel.did_you_mean(search_term)
}

pub fn generate_indices(
    language: &Language,
    db_path: &PathBuf,
    force: bool,
) -> Result<IndexingStream> {
    let channel = WiktionaryIngestChannel::init(language)?;
    let number_of_objects = channel.count()?;
    if number_of_objects > 0 && !force {
        bail!(
            "{} indices already exists for language '{}', use force to override",
            number_of_objects,
            &language
        );
    }
    match file_utils::get_file_reader(db_path) {
        Ok(file_reader) => Ok(IndexingStream::from(file_reader, channel)),
        _ => bail!("No such DB file: '{}'", db_path.display()),
    }
}
