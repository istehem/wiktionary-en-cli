use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use utilities::file_utils;
use utilities::language::*;

use wiktionary_en_db::wiktionary_en_db::*;

use clap::Parser;

/// Import Dictionary Data into PoloDB
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Override dictionary db file to use
    #[clap(long, short = 'd')]
    db_path: Option<String>,
    /// Language to import
    #[clap(long, short = 'l')]
    language: Option<String>,
    /// Force import, existing data will be overwritten
    #[clap(long, short = 'f')]
    force: bool,
    #[cfg(feature = "sonic")]
    /// Create identifier indices
    #[clap(long, short = 'i')]
    create_index: bool,
}

#[cfg(feature = "sonic")]
fn generate_identifier_indices(language: &Language, path: &PathBuf, force: bool) -> Result<()> {
    return wiktionary_en_identifier_index::generate_indices(language, path, force);
}

fn import_wiktionary_extract(path: &Path, language: &Language, force: bool) -> Result<()> {
    match file_utils::get_file_reader(path) {
        Ok(path) => return insert_wiktionary_file(path, language, force),
        Err(err) => bail!(err),
    }
}

fn get_language(language: &Option<String>) -> Result<Language> {
    if let Some(language) = language {
        return language.parse();
    }
    return Ok(Language::default());
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let language = get_language(&args.language)?;
    let db_path: PathBuf = get_db_path(args.db_path, &Some(language));
    #[cfg(feature = "sonic")]
    if args.create_index {
        return generate_identifier_indices(&language, &db_path, args.force);
    }
    return import_wiktionary_extract(&db_path, &language, args.force);
}
