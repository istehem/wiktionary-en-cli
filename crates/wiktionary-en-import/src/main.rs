use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use utilities::file_utils;
use utilities::language::*;

use wiktionary_en_db::wiktionary_en_db::*;

use clap::Parser;

/// Import Dictionary Data into Sonic
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
}

pub fn do_import(path: &Path, language: &Language, force: bool) -> Result<()> {
    match file_utils::get_file_reader(path) {
        Ok(path) => return insert_wiktionary_file(path, language, force),
        _ => bail!("No such DB file: '{}'", path.display()),
    }
}

fn get_language(language: &Option<String>) -> Language {
    if let Some(language) = language {
        return Language::from_string(&language).unwrap_or_default();
    }
    return Language::EN;
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let language = get_language(&args.language);
    let db_path: PathBuf = get_db_path(args.db_path, &Some(language));
    return do_import(&db_path, &language, args.force);
}
