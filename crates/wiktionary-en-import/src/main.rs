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
    /// A word to search for; omitting it will yield a random entry
    search_term: String,
    /// Override dictionary db file to use
    #[clap(long, short = 'd')]
    db_path: Option<String>,
    /// Language to import
    #[clap(long, short = 'l')]
    language: Option<String>,
    /// Force import even if data still exists in the bucket
    #[clap(long, short = 'f')]
    force: bool,
    /// Create indices
    #[clap(long, short = 'i')]
    create_indices: bool,
}

pub fn do_import(path: &Path, language: &Language) -> Result<()> {
    match file_utils::get_file_reader(path) {
        Ok(path) => return insert_wiktionary_file(path, language),
        _ => bail!("No such DB file: '{}'", path.display()),
    }
}

fn get_language(language: &Option<String>) -> Language {
    if let Some(language) = language {
        return Language::from_string(&language).unwrap_or_default();
    }
    return Language::EN;
}

pub fn find(term: &String, language: &Language, create_indices: bool) -> Result<()> {
    match find_by_word(term, language, create_indices) {
        Ok(entries) => {
            for entry in entries {
                println!("{}", entry.to_pretty_string());
            }
            return Ok(());
        }
        err @ Err(_) => err.map(|_| {}),
    }
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let language = get_language(&args.language);
    let db_path: PathBuf = get_db_path(args.db_path, &Some(language));
    if args.force {
        return do_import(&db_path, &language);
    } else {
        find(
            &args.search_term,
            &get_language(&args.language),
            args.create_indices,
        )?;
    }
    return Ok(());
}
