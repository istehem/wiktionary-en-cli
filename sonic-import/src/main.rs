use anyhow::{bail, Context, Result};
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use utilities::file_utils;
use utilities::file_utils::*;
use utilities::language::*;

use wiktionary_entities::wiktionary_entity::*;

use std::fs::File;

fn parse_line(line: Result<String, std::io::Error>, i: usize) -> Result<DictionaryEntry> {
    return line
        .map_err(|e| anyhow::Error::new(e).context(format!("Couldn't read line {} in DB file.", i)))
        .and_then(|line| {
            parse_entry(line).with_context(|| format!("Couldn't parse line {} in DB file.", i))
        });
}

fn parse_and_persist(file_reader: BufReader<File>) -> Result<()> {
    for (i, line) in file_reader.lines().enumerate() {
        let _dictionary_entry = parse_line(line, i);
    }
    return Ok(());
}

pub fn do_import(path: &Path) -> Result<()> {
    match file_utils::get_file_reader(path) {
        Ok(path) => return parse_and_persist(path),
        _ => bail!("No such DB file: '{}'", path.display()),
    }
}

fn get_db_path(path: Option<String>, language: &Option<Language>) -> PathBuf {
    if let Some(path) = path {
        return PathBuf::from(path);
    }
    return PathBuf::from(utilities::DICTIONARY_DB_PATH!(language
        .unwrap_or_default()
        .value()));
}

fn main() -> Result<()> {
    println!("{}", "Hello World!");
    println!("{}", env!("DICTIONARY_DIR"));
    println!(
        "{}",
        utilities::DICTIONARY_CACHING_PATH!(Language::EN.value())
    );
    println!("{}", utilities::DICTIONARY_DB_PATH!(Language::EN.value()));
    let db_path: PathBuf = get_db_path(None, &Some(Language::EN));
    return do_import(db_path.as_path());
}
