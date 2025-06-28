use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use utilities::file_utils;
use utilities::language::*;

use wiktionary_en_db::wiktionary_en_db::*;
use wiktionary_en_download::download_wiktionary_extract;

use clap::Parser;

#[cfg(feature = "sonic")]
mod indexing_executor;

/// Import And Download Wiktionary Data
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
    /// Download a wiktionary extract from the web
    #[clap(long, short = 'x')]
    download: bool,
}

fn import_wiktionary_extract(path: &Path, language: &Language, force: bool) -> Result<()> {
    match file_utils::get_file_reader(path) {
        Ok(path) => insert_wiktionary_file(path, language, force),
        Err(err) => bail!(err),
    }
}

fn get_language(language: &Option<String>) -> Result<Option<Language>> {
    if let Some(language) = language {
        return Ok(Some(language.parse()?));
    }
    Ok(None)
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let language = get_language(&args.language)?;
    let config_handler = wiktionary_en_lua::ConfigHandler::init()?;
    let language_to_use = language.unwrap_or(config_handler.config.language.unwrap_or_default());

    let db_path: PathBuf = get_db_path(args.db_path, &Some(language_to_use));
    #[cfg(feature = "sonic")]
    if args.create_index {
        let stream = wiktionary_en_identifier_index::generate_indices(
            &language_to_use,
            &db_path,
            args.force,
        )?;
        let errors = indexing_executor::execute_with_progress_bar_and_message(stream);
        return utilities::pager::print_lines_in_pager(&errors?);
    }
    if args.download {
        return download_wiktionary_extract(&language_to_use, args.force);
    }
    import_wiktionary_extract(&db_path, &language_to_use, args.force)
}
