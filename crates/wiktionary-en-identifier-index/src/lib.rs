use anyhow::{bail, Context, Result};
use std::io::{prelude::*, BufReader};
use std::path::PathBuf;
use utilities::file_utils;
use utilities::language::*;

use wiktionary_en_entities::wiktionary_entity::*;

use std::fmt;
use std::fs::File;

mod wiktionary_channel;
use crate::wiktionary_channel::*;

pub struct IndexingErrors {
    errors: Vec<IndexingError>,
}

impl fmt::Display for IndexingErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for error in &self.errors {
            writeln!(f, "{}", error)?;
        }
        return Ok(());
    }
}

pub struct IndexingError {
    iteration: usize,
    word: String,
    msg: String,
}

impl fmt::Display for IndexingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "failed to index '{}' after {} iterations with error {}",
            &self.word, &self.iteration, &self.msg
        )?;
        return Ok(());
    }
}

fn check_line(line: Result<String, std::io::Error>, i: usize) -> Result<String> {
    return line.map_err(|e| {
        anyhow::Error::new(e).context(format!("Couldn't read line {} in DB file.", i))
    });
}

fn parse_line(line: &String, i: usize) -> Result<DictionaryEntry> {
    parse_entry(line).with_context(|| format!("Couldn't parse line {} in DB file.", i))
}

fn parse_and_persist(
    channel: &WiktionaryIngestChannel,
    file_reader: BufReader<File>,
) -> Result<Vec<IndexingError>> {
    let flushb_count = channel.flush()?;
    dbg!(flushb_count);
    let mut count = 0;
    let mut errors = Vec::new();
    for (i, line) in file_reader.lines().enumerate() {
        let pushed = check_line(line, i).and_then(|line| {
            let dictionary_entry: DictionaryEntry = parse_line(&line, i)?;

            let push_result = channel.push(&dictionary_entry.word);
            if let Err(err) = push_result {
                let indexing_error = IndexingError {
                    iteration: count,
                    word: dictionary_entry.word.clone(),
                    msg: err.to_string(),
                };
                errors.push(indexing_error);
            }
            count = i;
            return Ok(());
        });
        if let Err(e) = pushed {
            bail!(e);
        }
    }
    println!("iterated over {} entries", count);
    return Ok(errors);
}

pub fn statistics(language: &Language) -> Result<()> {
    let ingest_channel = WiktionaryIngestChannel::init(language)?;
    return ingest_channel.statistics();
}

pub fn suggest(language: &Language, search_term: &String) -> Result<Vec<String>> {
    let search_channel = WiktionarySearchChannel::init(language)?;
    return search_channel.suggest(search_term);
}

pub fn query(language: &Language, search_term: &String) -> Result<Vec<String>> {
    let search_channel = WiktionarySearchChannel::init(language)?;
    return search_channel.query(search_term);
}

pub fn did_you_mean(language: &Language, search_term: &String) -> Result<Option<String>> {
    let search_channel = WiktionarySearchChannel::init(language)?;
    return search_channel.did_you_mean(search_term);
}

pub fn generate_indices(
    language: &Language,
    db_path: &PathBuf,
    force: bool,
) -> Result<IndexingErrors> {
    let channel = WiktionaryIngestChannel::init(language)?;
    let number_of_objects = channel.count()?;
    if number_of_objects > 0 && !force {
        bail!(
            "{} indices already exists, use force to override",
            number_of_objects
        );
    }
    match file_utils::get_file_reader(db_path) {
        Ok(path) => {
            return Ok(IndexingErrors {
                errors: parse_and_persist(&channel, path)?,
            })
        }
        _ => bail!("No such DB file: '{}'", db_path.display()),
    }
}
