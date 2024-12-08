use anyhow::{bail, Context, Result};
use clap::Parser;
use colored::Colorize;
use edit_distance::edit_distance;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{prelude::*, BufReader};
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

use minus::Pager;
use std::fmt::Write;

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

struct DidYouMean {
    searched_for: String,
    suggestion: String,
}

struct WiktionaryEnResult {
    did_you_mean: Option<DidYouMean>,
    hits: Vec<DictionaryEntry>,
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
        for hit in &self.hits {
            writeln!(f, "{}", &hit)?;
        }
        return Ok(());
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

fn did_you_mean_banner(search_term: &String, partial_match: &String) -> String {
    return formatdoc!(
        "
        No result for {}.
        Did you mean  {}?
        ",
        search_term.red(),
        partial_match.yellow()
    );
}

fn parse_line(line: Result<String, std::io::Error>, i: usize) -> Result<DictionaryEntry> {
    return line
        .map_err(|e| anyhow::Error::new(e).context(format!("Couldn't read line {} in DB file.", i)))
        .and_then(|line| {
            anyhow_serde::from_str(&line)
                .with_context(|| format!("Couldn't parse line {} in DB file.", i))
        });
}

fn print_stats(input_path_buf: PathBuf, language: &Language) -> Result<()> {
    let input_path = input_path_buf.as_path();
    if input_path.is_dir() {
        bail!(
            "specified wiktionary extract file '{}' is directory",
            input_path.display()
        );
    }

    println!(
        "{}",
        calculate_stats(input_path, &language).to_pretty_string()
    );

    return Ok(());
}

fn levenshtein_distance(search_term: &String, word: &String, case_insensitive: bool) -> usize {
    if case_insensitive {
        return edit_distance(
            &search_term.as_str().to_uppercase(),
            &word.as_str().to_uppercase(),
        );
    } else {
        return edit_distance(search_term, word);
    }
}

fn do_search(
    file_reader: BufReader<File>,
    term: String,
    max_results: usize,
    case_insensitive: bool,
) -> Result<SearchResult> {
    let search_result = search_worker(
        file_reader,
        term.clone(),
        max_results,
        case_insensitive,
        Arc::new(AtomicBool::new(false)),
    );
    return search_result;
}

fn evaluate_entry(
    search_result: &mut SearchResult,
    term: &String,
    json: DictionaryEntry,
    case_insensitive: bool,
    min_distance: usize,
) -> usize {
    let distance = levenshtein_distance(&json.word, &term, case_insensitive);
    if distance == 0 {
        search_result.full_matches.push(json.clone());
    }
    if distance < min_distance {
        search_result.did_you_mean = Some(json);
        search_result.distance = distance;
        return distance;
    }
    return min_distance;
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
    return Ok(search_result);
}

fn search(
    input_path: &PathBuf,
    term: String,
    max_results: usize,
    case_insensitive: bool,
) -> Result<SearchResult> {
    match get_file_reader(input_path.as_path()) {
        Ok(br) => return do_search(br, term, max_results, case_insensitive),
        Err(e) => bail!(e),
    }
}

fn find_by_word_in_db(term: &String, language: &Language) -> Result<Option<Vec<DictionaryEntry>>> {
    let db_hits = find_by_word(term, language);
    match db_hits {
        Ok(result) => {
            if result.is_empty() {
                return Ok(None);
            } else {
                return Ok(Some(result));
            }
        }
        Err(err) => bail!(err),
    }
}

fn run(
    term: &Option<String>,
    language: &Language,
    max_results: usize,
    case_insensitive: bool,
    path: PathBuf,
) -> Result<WiktionaryEnResult> {
    match term {
        Some(s) => match find_by_word_in_db(s, language) {
            Ok(Some(csr)) => {
                return Ok(WiktionaryEnResult {
                    did_you_mean: None,
                    hits: csr,
                });
            }
            Ok(None) => {
                #[cfg(feature = "sonic")]
                {
                    let did_you_mean = wiktionary_en_identifier_index::did_you_mean(language, s)?;
                    if let Some(did_you_mean) = did_you_mean {
                        let result = find_by_word_in_db(&did_you_mean, language)?;
                        if let Some(result) = result {
                            return Ok(WiktionaryEnResult {
                                did_you_mean: Some(DidYouMean {
                                    searched_for: s.to_string(),
                                    suggestion: did_you_mean,
                                }),
                                hits: result,
                            });
                        }
                    }
                }
                match search(&path, s.clone(), max_results, case_insensitive) {
                    Ok(sr) => {
                        if let Some(did_you_mean) = sr.did_you_mean {
                            return Ok(WiktionaryEnResult {
                                did_you_mean: Some(DidYouMean {
                                    searched_for: s.to_string(),
                                    suggestion: did_you_mean.word.clone(),
                                }),
                                hits: vec![did_you_mean],
                            });
                        }
                        return Ok(WiktionaryEnResult {
                            did_you_mean: None,
                            hits: sr.full_matches,
                        });
                    }
                    Err(e) => bail!(e),
                }
            }
            Err(e) => bail!(e),
        },
        None => {
            let hit = random_entry_for_language(language)?;
            return Ok(WiktionaryEnResult {
                did_you_mean: None,
                hits: vec![hit],
            });
        }
    };
}

fn get_language(language: &Option<String>) -> Result<Language> {
    if let Some(language) = language {
        return language.parse();
    }
    return Ok(Language::default());
}

fn get_db_path(path: Option<String>, language: &Language) -> PathBuf {
    if let Some(path) = path {
        return PathBuf::from(path);
    }
    return PathBuf::from(utilities::DICTIONARY_DB_PATH!(language.value()));
}

fn print_in_pager<T: std::fmt::Display>(value: &T) -> Result<()> {
    let mut output = Pager::new();
    writeln!(output, "{}", value)?;
    minus::page_all(output)?;
    return Ok(());
}

#[cfg(feature = "sonic")]
fn print_lines_in_pager<T: std::fmt::Display>(entries: &Vec<T>) -> Result<()> {
    let mut output = Pager::new();

    for entry in entries {
        writeln!(output, "{}", entry)?;
    }
    minus::page_all(output)?;
    return Ok(());
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let language = get_language(&args.language)?;

    if args.stats {
        return print_stats(get_db_path(args.db_path, &language), &language);
    }
    #[cfg(feature = "sonic")]
    if args.autocomplete {
        let search_term = &args
            .search_term
            .ok_or(anyhow::anyhow!("a search term is required"))?;
        let result = wiktionary_en_identifier_index::suggest(&language, search_term)?;
        print_lines_in_pager(&result)?;
        return Ok(());
    }
    #[cfg(feature = "sonic")]
    if args.query {
        let search_term = &args
            .search_term
            .ok_or(anyhow::anyhow!("a search term is required"))?;
        let result = wiktionary_en_identifier_index::query(&language, search_term)?;
        print_lines_in_pager(&result)?;
        return Ok(());
    }
    let result = run(
        &args.search_term,
        &language,
        args.max_results,
        args.case_insensitive,
        get_db_path(args.db_path, &language),
    )?;

    return print_in_pager(&result);
}
