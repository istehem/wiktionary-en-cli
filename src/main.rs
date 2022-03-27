use clap::Parser;
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

mod wiktionary_data;
use crate::wiktionary_data::*;

macro_rules! PROJECT_DIR {() => { env!("CARGO_MANIFEST_DIR")}; }

const DEFAULT_DB_SUB_PATH: &str = "files/wiktionary-en.json";
const DEFAULT_DB_PARTITIONED_DIR: &str = "files/partitioned";
const CHECK_FOR_SOLUTION_FOUND_EVERY : usize = 100;

/// An English Dictionary
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Override dictionary db file to use
    #[clap(long)]
    db_path: Option<String>,
    /// A word to search for; omitting it will yield a random entry
    search_term: Option<String>,
    /// Maximal number of results
    #[clap(short, long, default_value = "1")]
    max_results : usize,
    /// Use case insensitive search
    #[clap(short = 'i', long)]
    case_insensitive : bool,
    #[clap(short, long)]
    partitioned : bool
}

struct SearchResult {
   full_matches : Vec<DictionaryEntry>,
   did_you_mean : Option<DictionaryEntry>,
   distance     : usize
}

fn print_entry(json : &DictionaryEntry) {
    println!("{}", json.to_pretty_string());
}

fn print_search_result(term : String, search_result : SearchResult) {
    if search_result.full_matches.is_empty() {
        match search_result.did_you_mean {
            Some(result) => {
                printdoc!("
                          ###########################################
                          No result for {}.
                          Did you mean  {}?
                          ###########################################
                          ",
                          term.red(), &result.word.yellow());
                          print_entry(&result);
            },
            None         => println!("{}", "No results")
        }
    }
    for full_match in search_result.full_matches {
        print_entry(&full_match);
    }
}

fn get_file_reader(path: &Path) -> Result<BufReader<File>> {
    let file_buffer_result =  File::open(path)
        .map(|f| BufReader::new(f))
        .map_err(|err| anyhow::Error::new(err));
    match file_buffer_result {
        ok@Ok(_) => return ok,
        _        => bail!("No such DB file: '{}'", path.display().to_string())

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
    case_insensitive : bool) -> Result<()> {
    let search_result = search_worker(file_reader, term.clone(),
                            max_results, case_insensitive, Arc::new(AtomicBool::new(false)));
    match search_result {
        Ok(result) => print_search_result(term, result),
        Err(err)   => bail!(err)
    }
    return Ok(());
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
    case_insensitive : bool) -> Result<()> {
    let is_solution_found = Arc::new(AtomicBool::new(false));

    let paths = fs::read_dir(input_path).unwrap();
    let mut children = vec![];
    for path in paths {
        let term = term.clone();
        let max_results = max_results.clone();
        let case_insensitive_c = case_insensitive.clone();
        let is_solution_found = is_solution_found.clone();
        children.push(thread::spawn(move || {
            match get_file_reader(path.unwrap().path().as_path()) {
                Ok(br) => return search_worker(br, term, max_results,
                                               case_insensitive_c, is_solution_found),
                Err(e) => bail!(e)
            }
        }));
    }

    let mut search_results : Vec<SearchResult> = Vec::new();
    let mut did_you_mean : Option<DictionaryEntry> = None;
    let mut min_distance = usize::MAX;

    for child in children {
        let result = child.join().unwrap().unwrap();
        if result.distance < min_distance {
            min_distance = result.distance.clone();
            did_you_mean = result.did_you_mean.clone();
        }
        search_results.push(result);
    }

    let full_matches : Vec<DictionaryEntry> = search_results.into_iter()
                                                            .map(|r| r.full_matches)
                                                            .flatten()
                                                            .collect();
    print_search_result(term, SearchResult{
        full_matches : full_matches,
        did_you_mean : did_you_mean,
        distance     : min_distance
    });
    return Ok(());
}

fn search(input_path : &PathBuf, term : String, max_results : usize, case_insensitive : bool,
          partitioned : bool) -> Result<()> {
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

fn run(term : Option<String>, max_results : usize, case_insensitive : bool,
       partitioned : bool, path : PathBuf)
    -> Result<()> {
    match term {
       Some(s) => return search(&path, s, max_results, case_insensitive, partitioned),
       None    => return random_entry(&path.as_path())
    };
}

fn get_default_db_path(partitioned : bool, term : Option<String>) -> PathBuf {
    let mut path = PathBuf::from(PROJECT_DIR!());
    if partitioned && term.is_some() {
        path.push(DEFAULT_DB_PARTITIONED_DIR);
    }
    else {
        path.push(DEFAULT_DB_SUB_PATH);
    }
    return path;
}

fn main() -> Result<()> {
    let args = Cli::parse();
    match args.db_path {
       Some(path) => return run(args.search_term, args.max_results,
                                args.case_insensitive, args.partitioned, PathBuf::from(path)),
       None       => return run(args.search_term.clone(), args.max_results,
                                args.case_insensitive, args.partitioned,
                                    get_default_db_path(args.partitioned, args.search_term))
    };
}
