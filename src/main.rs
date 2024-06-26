use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use colored::Colorize;
use edit_distance::edit_distance;
use indoc::printdoc;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

use utilities::anyhow_serde;
use utilities::file_utils::*;
use utilities::language::*;

mod wiktionary_data;
use crate::wiktionary_data::*;
mod wiktionary_stats;
use crate::wiktionary_stats::*;
mod wiktionary_cache;
use crate::wiktionary_cache::*;

macro_rules! DICTIONARY_DB_PATH {
    ($language:expr) => {
        format!("{}/wiktionary-{}.json", env!("DICTIONARY_DIR"), $language)
    };
}
macro_rules! DEFAULT_DB_PARTITIONED_DIR {
    () => {
        format!("{}/partitioned", env!("DICTIONARY_DIR"))
    };
}

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
    /// Set search term language (ignored when used with --db-path)
    #[clap(long, short = 'l')]
    language: Option<String>,
    #[clap(short, long)]
    partitioned: bool,
    /// Show dictionary information
    #[clap(short, long)]
    stats: bool,
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

fn print_entry(json: &DictionaryEntry) {
    println!("{}", json.to_pretty_string());
}

fn print_search_result(term: &String, search_result: &SearchResult) {
    if search_result.full_matches.is_empty() {
        match &search_result.did_you_mean {
            Some(result) => {
                printdoc!(
                    "
                          No result for {}.
                          Did you mean  {}?
                          ",
                    term.red(),
                    &result.word.yellow()
                );
                print_entry(&result);
            }
            None => println!("{}", "No results"),
        }
    }
    for full_match in &search_result.full_matches {
        print_entry(&full_match);
    }
}

fn print_entries(entries: &Vec<DictionaryEntry>) {
    for entry in entries {
        print_entry(&entry);
    }
}

fn find_entry(file_reader: BufReader<File>, index: usize) -> Result<DictionaryEntry> {
    for (i, line) in file_reader.lines().enumerate() {
        if i == index {
            return parse_line(line, i);
        }
    }
    bail!("No entry found.");
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
        bail!("Sorry, cannot calculate stats for partitioned search yet");
    }

    println!(
        "{}",
        calculate_stats(
            input_path,
            &wiktionary_cache::DICTIONARY_CACHING_PATH!(language.value())
        )
        .to_pretty_string()
    );

    return Ok(());
}

fn random_entry(input_path: &Path) -> Result<()> {
    let file_reader = get_file_reader(input_path);
    let n_entries = file_reader.map(|file_reader| file_reader.lines().count());
    let random_entry_number = n_entries.map(|n| thread_rng().gen_range(0, n - 1));

    match random_entry_number {
        Ok(random_entry_number) => {
            let result = get_file_reader(input_path)
                .and_then(|file_reader| find_entry(file_reader, random_entry_number));
            return result.map(|entry| {
                print_entry(&entry);
                return;
            });
        }
        Err(err) => bail!(err.context("Couldn't generate random entry number.")),
    }
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

fn search_partitioned(
    input_path: &PathBuf,
    term: String,
    max_results: usize,
    case_insensitive: bool,
) -> Result<SearchResult> {
    let is_solution_found = Arc::new(AtomicBool::new(false));

    let mut children = vec![];
    let paths = fs::read_dir(input_path);
    ensure!(
        paths.is_ok(),
        format!("Couldn't find db dir: '{}'", input_path.display())
    );
    for path in paths.unwrap() {
        let term = term.clone();
        let max_results = max_results.clone();
        let case_insensitive_c = case_insensitive.clone();
        let is_solution_found = is_solution_found.clone();
        children.push(thread::spawn(move || {
            if let Ok(path) = path {
                match get_file_reader(path.path().as_path()) {
                    Ok(br) => {
                        return search_worker(
                            br,
                            term,
                            max_results,
                            case_insensitive_c,
                            is_solution_found,
                        )
                    }
                    Err(e) => bail!(e),
                }
            }
            bail!("db file path contains an invalid partition entry");
        }));
    }

    let mut search_results: Vec<SearchResult> = Vec::new();
    let mut did_you_mean: Option<DictionaryEntry> = None;
    let mut min_distance = usize::MAX;

    for child in children {
        if let Ok(child_join) = child.join() {
            match child_join {
                Ok(result) => {
                    if result.distance < min_distance {
                        min_distance = result.distance.clone();
                        did_you_mean = result.did_you_mean.clone();
                    }
                    search_results.push(result)
                }
                Err(err) => bail!(err),
            }
        } else {
            bail!("thread panicked!");
        }
    }

    let full_matches: Vec<DictionaryEntry> = search_results
        .into_iter()
        .map(|r| r.full_matches)
        .flatten()
        .collect();
    let search_result = SearchResult {
        full_matches: full_matches,
        did_you_mean: did_you_mean,
        distance: min_distance,
    };
    return Ok(search_result);
}

fn search(
    input_path: &PathBuf,
    term: String,
    max_results: usize,
    case_insensitive: bool,
    partitioned: bool,
) -> Result<SearchResult> {
    if partitioned {
        return search_partitioned(input_path, term, max_results, case_insensitive);
    } else {
        match get_file_reader(input_path.as_path()) {
            Ok(br) => return do_search(br, term, max_results, case_insensitive),
            Err(e) => bail!(e),
        }
    }
}

fn evaluate_cache(
    term: &String,
    max_results: usize,
    language: &Language,
) -> Result<Option<Vec<DictionaryEntry>>> {
    match get_cached_entries(term, language) {
        Ok(Some(result)) => {
            if result.len() < max_results {
                return Ok(None);
            } else {
                return Ok(Some(result));
            }
        }
        result => result,
    }
}

fn run(
    term: &Option<String>,
    language: &Language,
    max_results: usize,
    case_insensitive: bool,
    partitioned: bool,
    path: PathBuf,
) -> Result<()> {
    match term {
        Some(s) => match evaluate_cache(s, max_results, language) {
            Ok(Some(csr)) => {
                print_entries(&csr);
                return Ok(());
            }
            Ok(None) => {
                match search(&path, s.clone(), max_results, case_insensitive, partitioned) {
                    Ok(sr) => {
                        print_search_result(s, &sr);
                        if !sr.full_matches.is_empty() {
                            return write_to_cache(s, &sr.full_matches, language);
                        }
                        return Ok(());
                    }
                    Err(e) => bail!(e),
                }
            }
            Err(e) => bail!(e),
        },
        None => return random_entry(&path.as_path()),
    };
}

fn get_language(language: &Option<String>) -> Language {
    if let Some(language) = language {
        return Language::from_string(&language).unwrap_or_default();
    }
    return Language::EN;
}

fn get_db_path(
    path: Option<String>,
    language: &Option<String>,
    partitioned: bool,
    search_term: &Option<String>,
) -> PathBuf {
    if let Some(path) = path {
        return PathBuf::from(path);
    }

    if partitioned && search_term.is_some() {
        return PathBuf::from(DEFAULT_DB_PARTITIONED_DIR!());
    }

    return PathBuf::from(DICTIONARY_DB_PATH!(get_language(language).value()));
}

fn write_to_cache(term: &String, value: &Vec<DictionaryEntry>, language: &Language) -> Result<()> {
    let entry = CachedDbEntry {
        entries: value.clone(),
    };
    return write_db_entry_to_cache(term, &entry, language);
}

fn get_cached_entries(term: &String, language: &Language) -> Result<Option<Vec<DictionaryEntry>>> {
    return wiktionary_cache::get_cached_db_entry(term, language)
        .map(|cde: Option<CachedDbEntry>| cde.map(|cde| cde.entries));
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let language = get_language(&args.language);
    match args.stats {
        true => {
            return print_stats(
                get_db_path(
                    args.db_path,
                    &args.language,
                    args.partitioned,
                    &args.search_term,
                ),
                &language,
            )
        }
        _ => {
            return run(
                &args.search_term,
                &language,
                args.max_results,
                args.case_insensitive,
                args.partitioned,
                get_db_path(
                    args.db_path,
                    &args.language,
                    args.partitioned,
                    &args.search_term,
                ),
            )
        }
    };
}
