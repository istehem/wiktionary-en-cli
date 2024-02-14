use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::{Path, PathBuf};
use anyhow::{Result, bail, ensure};
use rand::{thread_rng, Rng};
use indoc::{printdoc};
use colored::Colorize;
use std::env;
use edit_distance::edit_distance;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::thread;
use std::sync::atomic::Ordering;

use utilities::file_utils::*;
use utilities::language::*;

mod wiktionary_data;
use crate::wiktionary_data::*;
mod wiktionary_stats;
use crate::wiktionary_stats::*;

macro_rules! PROJECT_DIR{ () => { env!("CARGO_MANIFEST_DIR")}; }
macro_rules! DICTIONARY_DB_SUB_PATH { ($language:tt) => { format!("files/wiktionary-{}.json", $language)}; }
macro_rules! DICTIONARY_CACHING_PATH { ($language:expr) => { format!("{}/cache/wiktionary-cache-{}", PROJECT_DIR!(), $language)}; }
macro_rules! DEFAULT_DB_SUB_PATH { () => { DICTIONARY_DB_SUB_PATH!("en")}; }

const DEFAULT_DB_PARTITIONED_DIR: &str = "files/partitioned";
const CHECK_FOR_SOLUTION_FOUND_EVERY : usize = 100;

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
    max_results : usize,
    /// Use case insensitive search
    #[clap(short = 'i', long)]
    case_insensitive : bool,
    /// Set search term language (ignored when used with --db-path)
    #[clap(long, short = 'l')]
    language: Option<String>,
    #[clap(short, long)]
    partitioned : bool,
    /// Show dictionary information
    #[clap(short, long)]
    stats : bool
}

struct SearchResult {
   full_matches : Vec<DictionaryEntry>,
   did_you_mean : Option<DictionaryEntry>,
   distance     : usize
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CachedDbEntry {
   entries  : Vec<DictionaryEntry>
}
impl CachedDbEntry {
    pub fn to_json(&self) -> Result<String> {
        return serde_json::to_string(self).map_err(|err| anyhow::Error::new(err));
    }
}

fn print_entry(json : &DictionaryEntry) {
    println!("{}", json.to_pretty_string());
}

fn print_search_result(term : &String, search_result : &SearchResult) {
    if search_result.full_matches.is_empty() {
        match &search_result.did_you_mean {
            Some(result) => {
                printdoc!("
                          No result for {}.
                          Did you mean  {}?
                          ",
                          term.red(), &result.word.yellow());
                          print_entry(&result);
            },
            None         => println!("{}", "No results")
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

fn find_entry(file_reader : BufReader<File>, index : usize) -> Result<DictionaryEntry> {
    for (i, line) in file_reader.lines().enumerate() {
        if i == index {
            return parse_line(line, i);
        }
    }
    bail!("No entry found.");
}

fn parse_line(line : Result<String, std::io::Error>, i : usize) -> Result<DictionaryEntry> {
    ensure!(line.is_ok(), format!("Couldn't read line {} in DB file.", i));
    let parse_res : Result<DictionaryEntry> = wiktionary_data::parse(&line?);
    ensure!(parse_res.is_ok(), format!("Couldn't parse line {} in DB file.", i));
    return parse_res;
}

fn print_stats(input_path_buf : PathBuf) -> Result<()> {
    let input_path = input_path_buf.as_path();
    if input_path.is_dir(){
        bail!("Sorry, cannot calculate stats for partitioned search yet");
    }

    println!("{}", calculate_stats(input_path).to_pretty_string());

    return Ok(());
}

fn random_entry(input_path : &Path) -> Result<()> {
    let file_reader = get_file_reader(input_path);
    ensure!(file_reader.is_ok(), file_reader.unwrap_err());
    let n_entries : Option<usize> = file_reader.ok().map(|br| br.lines().count());
    let mut rng = thread_rng();
    let random_entry_number: Option<usize> =
        n_entries.map(|n| rng.gen_range(0, n - 1));
    match get_file_reader(input_path)
        .ok()
        .zip(random_entry_number)
        .map(|(br, index)| find_entry(br, index))
        .transpose() {
        Ok(Some(json)) => print_entry(&json),
        Ok(None)       => bail!("Couldn't generate random entry number."),
        Err(err)       => bail!(err)
    }
    return Ok(());
}

fn levenshtein_distance(search_term : &String, word : &String, case_insensitive : bool)
    -> usize {
    if case_insensitive {
        return edit_distance(&search_term.as_str().to_uppercase(),
                             &word.as_str().to_uppercase());
    }
    else {
        return edit_distance(search_term, word);
    }
}

fn do_search(file_reader : BufReader<File>, term : String, max_results : usize,
    case_insensitive : bool) -> Result<SearchResult> {
    let search_result = search_worker(file_reader, term.clone(),
                            max_results, case_insensitive, Arc::new(AtomicBool::new(false)));
    return search_result;
}

fn search_worker(file_reader : BufReader<File>, term : String, max_results : usize,
    case_insensitive : bool, is_solution_found: Arc<AtomicBool>)
    -> Result<SearchResult> {
    let mut search_result = SearchResult {
        full_matches : Vec::new(),
        did_you_mean : None,
        distance     : usize::MAX
    };
    let mut min_distance = usize::MAX;
    for (i, line) in file_reader.lines().enumerate() {
        let parse_res : Result<DictionaryEntry> = parse_line(line, i);
        ensure!(parse_res.is_ok(), parse_res.unwrap_err());
        let json : DictionaryEntry = parse_res?;
        let distance = levenshtein_distance(&json.word, &term, case_insensitive);
        if distance < min_distance {
            min_distance = distance;
            search_result.did_you_mean = Some(json.clone());
            search_result.distance = distance;
        }
        if distance == 0 {
            search_result.full_matches.push(json);
        }
        if search_result.full_matches.len() == max_results {
            is_solution_found.store(true, Ordering::Relaxed);
            break;
        }
        if i % CHECK_FOR_SOLUTION_FOUND_EVERY == 0
            && is_solution_found.load(Ordering::Relaxed) {
            break;
        }
    }
    return Ok(search_result);
}

fn search_partitioned(input_path : &PathBuf, term : String, max_results : usize,
    case_insensitive : bool) -> Result<SearchResult> {
    let is_solution_found = Arc::new(AtomicBool::new(false));

    let mut children = vec![];
    let paths = fs::read_dir(input_path);
    ensure!(paths.is_ok(), format!("Couldn't find db dir: '{}'", input_path.display()));
    for path in paths.unwrap() {
        let term = term.clone();
        let max_results = max_results.clone();
        let case_insensitive_c = case_insensitive.clone();
        let is_solution_found = is_solution_found.clone();
        children.push(thread::spawn(move || {
            if let Ok(path) = path {
                match get_file_reader(path.path().as_path()) {
                    Ok(br) => return search_worker(br, term, max_results,
                                               case_insensitive_c, is_solution_found),
                    Err(e) => bail!(e)
                }
            }
            bail!("db file path contains an invalid partition entry");
        }));
    }

    let mut search_results : Vec<SearchResult> = Vec::new();
    let mut did_you_mean : Option<DictionaryEntry> = None;
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
                },
            Err(err) => bail!(err)
            }
        }
        else {
            bail!("thread panicked!");
        }
    }

    let full_matches : Vec<DictionaryEntry> = search_results.into_iter()
                                                            .map(|r| r.full_matches)
                                                            .flatten()
                                                            .collect();
    let search_result =
        SearchResult {
            full_matches : full_matches,
            did_you_mean : did_you_mean,
            distance     : min_distance
        };
    return Ok(search_result);
}

fn search(input_path : &PathBuf, term : String, max_results : usize, case_insensitive : bool,
          partitioned : bool) -> Result<SearchResult> {
    if partitioned {
        return search_partitioned(input_path, term, max_results, case_insensitive);
    }
    else {
        match get_file_reader(input_path.as_path()) {
            Ok(br) => return do_search(br, term, max_results, case_insensitive),
            Err(e) => bail!(e)
        }
    }
}

fn run(term : &Option<String>, language : &Language, max_results : usize,
    case_insensitive : bool, partitioned : bool, path : PathBuf)
    -> Result<()> {
    match term {
       Some(s) => {
            match get_cached_db_entry(s, language) {
                Ok(csr) => {
                    print_entries(&csr);
                    return Ok(());
                }
                _       => match search(&path, s.clone(), max_results, case_insensitive,
                             partitioned) {
                                Ok(sr) => {
                                 print_search_result(s, &sr);
                                 return write_db_entry_to_cache(s, &sr.full_matches, language)
                            },
                            Err(e) => bail!(e)
                            }
            }
       },
       None    => return random_entry(&path.as_path())
    };
}

fn get_language(language : &Option<String>) -> Language {
    if let Some(language) = language {
       return Language::from_string(&language).unwrap_or(Language::EN);
    }
    return Language::EN;
}

fn get_db_path(path_buf: Option<String>, language: &Option<String>,
    partitioned: bool, search_term: &Option<String>) -> PathBuf {

    if let Some(path_buf) = path_buf {
        return PathBuf::from(path_buf);
    }

    let mut path = PathBuf::from(PROJECT_DIR!());

    if partitioned && search_term.is_some() {
        path.push(DEFAULT_DB_PARTITIONED_DIR);
    }
    else {
        path.push(DICTIONARY_DB_SUB_PATH!((get_language(language).value())));
    }
    return path;
}

fn write_db_entry_to_cache(term: &String, value: &Vec<DictionaryEntry>, language: &Language)
    -> Result<()> {
    let value_as_json: String =
        CachedDbEntry {entries: value.clone()}.to_json().unwrap();

    // this directory will be created if it does not exist
    let path = DICTIONARY_CACHING_PATH!(language.value());

    // works like std::fs::open
    let db = sled::open(path)?;

    // key and value types can be `Vec<u8>`, `[u8]`, or `str`.
    let key = term;

    // `generate_id`
    // let value = db.generate_id()?.to_be_bytes();

    //dbg!(
    db.insert(key, value_as_json.as_bytes()); // as in BTreeMap::insert
    db.get(key)?;                // as in BTreeMap::get
    //db.remove(key)?,             // as in BTreeMap::remove
    //);

    Ok(())
}

fn get_cached_db_entry(term: &String, language: &Language) -> Result<Vec<DictionaryEntry>> {
    let path = DICTIONARY_CACHING_PATH!(language.value());

    let db = sled::open(path)?;

    match db.get(term){
        Ok(Some(b)) => return String::from_utf8((&b).to_vec())
            .map_err(|err| anyhow::Error::new(err))
            .and_then(|s| parse(&s))
            .map(|cde| cde.entries),
        Ok(_)       => bail!("entry not found"),
        Err(err)    => bail!(err)
    };
}

pub fn parse(line : &String) -> Result<CachedDbEntry> {
    return serde_json::from_str(line).map_err(|err| anyhow::Error::new(err));
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.stats {
        true =>
           return print_stats(get_db_path(args.db_path, &args.language, args.partitioned,
                                         &args.search_term)),
        _   =>
           return run(&args.search_term, &get_language(&args.language), args.max_results,
                      args.case_insensitive, args.partitioned,
                      get_db_path(args.db_path, &args.language, args.partitioned,
                                  &args.search_term))
    };
}
