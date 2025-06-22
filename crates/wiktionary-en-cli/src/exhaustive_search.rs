use anyhow::{bail, Context, Result};
use edit_distance::edit_distance;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use utilities::anyhow_serde;
use utilities::file_utils::*;
use wiktionary_en_entities::wiktionary_entry::*;

const CHECK_FOR_SOLUTION_FOUND_EVERY: usize = 100;

pub struct SearchResult {
    pub full_matches: Vec<DictionaryEntry>,
    pub did_you_mean: Option<DictionaryEntry>,
    distance: usize,
}

fn levenshtein_distance(search_term: &str, word: &str, case_insensitive: bool) -> usize {
    if case_insensitive {
        edit_distance(&search_term.to_uppercase(), &word.to_uppercase())
    } else {
        edit_distance(search_term, word)
    }
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

fn parse_line(line: Result<String, std::io::Error>, i: usize) -> Result<DictionaryEntry> {
    line.map_err(|e| anyhow::Error::new(e).context(format!("Couldn't read line {} in DB file.", i)))
        .and_then(|line| {
            anyhow_serde::from_str(&line)
                .with_context(|| format!("Couldn't parse line {} in DB file.", i))
        })
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

pub fn search(
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
