use anyhow::{bail, Context, Result};
use clap::Parser;
use colored::Colorize;
use edit_distance::edit_distance;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use utilities::anyhow_serde;
use utilities::file_utils::*;
use utilities::language::*;

use wiktionary_en_entities::wiktionary_entity::*;

mod wiktionary_stats;
use crate::wiktionary_stats::*;

use wiktionary_en_db::wiktionary_en_db::*;

use std::fmt;

const CHECK_FOR_SOLUTION_FOUND_EVERY: usize = 100;

/// A To English Dictionary
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Override dictionary db file to use
    #[clap(long, short = 'd')]
    db_path: Option<String>,
    /// A word to search for; omitting it will yield a random entry
    search_term: Option<String>,
    /// Maximal number of results
    #[clap(short, long, default_value = "1")]
    max_results: usize,
    /// Use case insensitive search
    #[clap(short = 'i', long)]
    case_insensitive: bool,
    /// Set search term language
    #[clap(long, short = 'l')]
    language: Option<String>,
    /// Show dictionary information
    #[clap(short, long)]
    stats: bool,
    #[cfg(feature = "sonic")]
    /// Autocomplete word
    #[clap(short, long)]
    autocomplete: bool,
    #[cfg(feature = "sonic")]
    /// Query word
    #[clap(short, long)]
    query: bool,
}

struct QueryParameters {
    search_term: Option<String>,
    language: Language,
    max_results: usize,
    case_insensitive: bool,
    path: PathBuf,
}

struct DidYouMean {
    searched_for: String,
    suggestion: String,
}

struct WiktionaryEnResult {
    did_you_mean: Option<DidYouMean>,
    hits: Vec<DictionaryEntry>,
    config_handler: wiktionary_en_lua::ConfigHandler,
}

impl WiktionaryEnResult {
    pub fn intercept(&mut self) -> Result<()> {
        if let Some(hits) = self
            .config_handler
            .intercept_wiktionary_result(&self.hits)?
        {
            self.hits = hits;
        }
        Ok(())
    }
}

impl fmt::Display for WiktionaryEnResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(did_you_mean) = &self.did_you_mean {
            writeln!(
                f,
                "{}",
                did_you_mean_banner(&did_you_mean.searched_for, &did_you_mean.suggestion)
            )?;
        }

        match self.config_handler.format_wiktionary_result(&self.hits) {
            Ok(Some(formated_hits)) => {
                for hit in &formated_hits {
                    writeln!(f, "{}", &hit)?;
                }
                Ok(())
            }
            Ok(None) => {
                for hit in &self.hits {
                    writeln!(f, "{}", &hit)?;
                }
                Ok(())
            }
            Err(err) => {
                eprintln!("{:?}", err);
                Err(fmt::Error)
            }
        }
    }
}

struct SearchResult {
    full_matches: Vec<DictionaryEntry>,
    did_you_mean: Option<DictionaryEntry>,
    distance: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedDbEntry {
    entries: Vec<DictionaryEntry>,
}

fn did_you_mean_banner(search_term: &str, partial_match: &str) -> String {
    formatdoc!(
        "
        No result for {}.
        Did you mean  {}?
        ",
        search_term.red(),
        partial_match.yellow()
    )
}

fn parse_line(line: Result<String, std::io::Error>, i: usize) -> Result<DictionaryEntry> {
    line.map_err(|e| anyhow::Error::new(e).context(format!("Couldn't read line {} in DB file.", i)))
        .and_then(|line| {
            anyhow_serde::from_str(&line)
                .with_context(|| format!("Couldn't parse line {} in DB file.", i))
        })
}

fn print_stats(input_path_buf: PathBuf, language: &Language) -> Result<()> {
    let input_path = input_path_buf.as_path();
    if input_path.is_dir() {
        bail!(
            "specified wiktionary extract file '{}' is a directory",
            input_path.display()
        );
    }

    println!(
        "{}",
        calculate_stats(input_path, language).to_pretty_string()
    );

    Ok(())
}

fn levenshtein_distance(search_term: &str, word: &str, case_insensitive: bool) -> usize {
    if case_insensitive {
        edit_distance(&search_term.to_uppercase(), &word.to_uppercase())
    } else {
        edit_distance(search_term, word)
    }
}

fn do_search(
    file_reader: BufReader<File>,
    term: String,
    max_results: usize,
    case_insensitive: bool,
) -> Result<SearchResult> {
    search_worker(
        file_reader,
        term.clone(),
        max_results,
        case_insensitive,
        Arc::new(AtomicBool::new(false)),
    )
}

fn evaluate_entry(
    search_result: &mut SearchResult,
    term: &str,
    json: DictionaryEntry,
    case_insensitive: bool,
    min_distance: usize,
) -> usize {
    let distance = levenshtein_distance(&json.word, term, case_insensitive);
    if distance == 0 {
        search_result.full_matches.push(json.clone());
    }
    if distance < min_distance {
        search_result.did_you_mean = Some(json);
        search_result.distance = distance;
        return distance;
    }
    min_distance
}

fn search_worker(
    file_reader: BufReader<File>,
    term: String,
    max_results: usize,
    case_insensitive: bool,
    is_solution_found: Arc<AtomicBool>,
) -> Result<SearchResult> {
    let mut search_result = SearchResult {
        full_matches: Vec::new(),
        did_you_mean: None,
        distance: usize::MAX,
    };
    let mut min_distance = usize::MAX;
    for (i, line) in file_reader.lines().enumerate() {
        let parse_res: Result<DictionaryEntry> = parse_line(line, i);

        match parse_res {
            Ok(json) => {
                min_distance = evaluate_entry(
                    &mut search_result,
                    &term,
                    json,
                    case_insensitive,
                    min_distance,
                )
            }
            Err(err) => bail!(err),
        }

        if search_result.full_matches.len() == max_results {
            is_solution_found.store(true, Ordering::Relaxed);
            break;
        }
        if i % CHECK_FOR_SOLUTION_FOUND_EVERY == 0 && is_solution_found.load(Ordering::Relaxed) {
            break;
        }
    }
    Ok(search_result)
}

fn search(
    input_path: &Path,
    term: String,
    max_results: usize,
    case_insensitive: bool,
) -> Result<SearchResult> {
    match get_file_reader(input_path) {
        Ok(br) => do_search(br, term, max_results, case_insensitive),
        Err(e) => bail!(e),
    }
}

fn find_by_word_in_db(term: &String, language: &Language) -> Result<Option<Vec<DictionaryEntry>>> {
    let db_hits = find_by_word(term, language);
    match db_hits {
        Ok(result) => {
            if result.is_empty() {
                Ok(None)
            } else {
                Ok(Some(result))
            }
        }
        Err(err) => bail!(err),
    }
}

fn run(
    query_params: QueryParameters,
    config_handler: wiktionary_en_lua::ConfigHandler,
) -> Result<WiktionaryEnResult> {
    match query_params.search_term {
        Some(s) => match find_by_word_in_db(&s, &query_params.language) {
            Ok(Some(csr)) => Ok(WiktionaryEnResult {
                did_you_mean: None,
                hits: csr,
                config_handler,
            }),
            Ok(None) => {
                #[cfg(feature = "sonic")]
                {
                    let did_you_mean =
                        wiktionary_en_identifier_index::did_you_mean(&query_params.language, &s)?;
                    if let Some(did_you_mean) = did_you_mean {
                        let result = find_by_word_in_db(&did_you_mean, &query_params.language)?;
                        if let Some(result) = result {
                            return Ok(WiktionaryEnResult {
                                did_you_mean: Some(DidYouMean {
                                    searched_for: s.to_string(),
                                    suggestion: did_you_mean,
                                }),
                                hits: result,
                                config_handler: config_handler,
                            });
                        }
                    }
                }
                match search(
                    &query_params.path,
                    s.clone(),
                    query_params.max_results,
                    query_params.case_insensitive,
                ) {
                    Ok(sr) => {
                        if let Some(did_you_mean) = sr.did_you_mean {
                            return Ok(WiktionaryEnResult {
                                did_you_mean: Some(DidYouMean {
                                    searched_for: s.to_string(),
                                    suggestion: did_you_mean.word.clone(),
                                }),
                                hits: vec![did_you_mean],
                                config_handler,
                            });
                        }
                        Ok(WiktionaryEnResult {
                            did_you_mean: None,
                            hits: sr.full_matches,
                            config_handler,
                        })
                    }
                    Err(e) => bail!(e),
                }
            }
            Err(e) => bail!(e),
        },
        None => {
            let hit = random_entry_for_language(&query_params.language)?;
            Ok(WiktionaryEnResult {
                did_you_mean: None,
                hits: vec![hit],
                config_handler,
            })
        }
    }
}

fn get_language(language: &Option<String>) -> Result<Option<Language>> {
    if let Some(language) = language {
        return Ok(Some(language.parse()?));
    }
    Ok(None)
}

fn get_db_path(path: Option<String>, language: &Language) -> PathBuf {
    if let Some(path) = path {
        return PathBuf::from(path);
    }
    PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()))
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let language = get_language(&args.language)?;
    let config_handler = wiktionary_en_lua::ConfigHandler::init()?;
    let language_to_use = language.unwrap_or(config_handler.config.language.unwrap_or_default());

    if args.stats {
        return print_stats(
            get_db_path(args.db_path, &language_to_use),
            &language_to_use,
        );
    }
    #[cfg(feature = "sonic")]
    if args.autocomplete {
        let search_term = &args
            .search_term
            .ok_or(anyhow::anyhow!("a search term is required"))?;
        let result = wiktionary_en_identifier_index::suggest(&language_to_use, search_term)?;
        utilities::pager::print_lines_in_pager(&result)?;
        return Ok(());
    }
    #[cfg(feature = "sonic")]
    if args.query {
        let search_term = &args
            .search_term
            .ok_or(anyhow::anyhow!("a search term is required"))?;
        let result = wiktionary_en_identifier_index::query(&language_to_use, search_term)?;
        utilities::pager::print_lines_in_pager(&result)?;
        return Ok(());
    }

    let mut result = run(
        QueryParameters {
            search_term: args.search_term,
            language: language_to_use,
            max_results: args.max_results,
            case_insensitive: args.case_insensitive,
            path: get_db_path(args.db_path, &language_to_use),
        },
        config_handler,
    )?;
    result.intercept()?;
    utilities::pager::print_in_pager(&result)
}
