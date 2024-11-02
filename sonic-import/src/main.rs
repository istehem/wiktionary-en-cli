use anyhow::{bail, Context, Result};
use std::path::Path;
use utilities::anyhow_serde;
use utilities::file_utils;
use utilities::language::*;

use wiktionary_entities::wiktionary_entity::*;

fn parse_line(line: Result<String, std::io::Error>, i: usize) -> Result<DictionaryEntry> {
    return line
        .map_err(|e| anyhow::Error::new(e).context(format!("Couldn't read line {} in DB file.", i)))
        .and_then(|line| {
            parse_entry(line).with_context(|| format!("Couldn't parse line {} in DB file.", i))
        });
}

pub fn do_import(path: &Path) -> Result<()> {
    match file_utils::get_file_reader(path) {
        Ok(_) => return Ok(()),
        _ => bail!("No such DB file: '{}'", path.display()),
    }
}

fn main() -> Result<()> {
    println!("{}", "Hello World!");
    println!("{}", env!("DICTIONARY_DIR"));
    println!(
        "{}",
        utilities::DICTIONARY_CACHING_PATH!(Language::EN.value())
    );
    println!("{}", utilities::DICTIONARY_DB_PATH!(Language::EN.value()));
    return Ok(());
}
