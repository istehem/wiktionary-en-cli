use clap::Parser;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;
use anyhow::{Result, bail, ensure};
use rand::{thread_rng, Rng};
use indoc::{printdoc};
use colored::Colorize;
use std::env;
use edit_distance::edit_distance;
use std::fs;
//use std::sync::mpsc::{Sender, Receiver};
//use std::sync::mpsc;
use std::thread;

mod wiktionary_data;
use crate::wiktionary_data::*;

static DEFAULT_DB_SUB_PATH: &str = "files/wiktionary-en.json";

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

fn print_entry(json : &DictionaryEntry) {
    println!("{}", json.to_pretty_string());
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
    let mut result : Option<DictionaryEntry> = None;
    let mut full_matches : Vec<DictionaryEntry> = Vec::new();
    let mut min_distance = usize::MAX;
    for (i, line) in file_reader.lines().enumerate() {
        let parse_res : Result<DictionaryEntry> = parse_line(line, i);
        ensure!(parse_res.is_ok(), parse_res.unwrap_err());
        let json : DictionaryEntry = parse_res?;
        let distance = levenshtein_distance(&json.word, &term, case_insensitive);
        if distance < min_distance {
            min_distance = distance;
            result = Some(json.clone());
        }
        if distance == 0 {
            full_matches.push(json);
        }
        if full_matches.len() == max_results {
            break;
        }
    }
    match result {
        None        => println!("{}", "No results"),
        Some(json)  => {
            if min_distance != 0 {
                printdoc!("
                          ###########################################
                          No result for {}.
                          Did you mean  {}?
                          ###########################################
                          ",
                          &term.red(), &json.word.yellow());
                          print_entry(&json);
            }
            for res in full_matches {
                print_entry(&res);
            }
        }
    };
    return Ok(());
}

fn search_partitioned(input_path : &Path, term : String, max_results : usize,
    case_insensitive : bool) -> Result<()> {
    let paths = fs::read_dir("files/partitioned").unwrap();
    let mut children = vec![];
    for path in paths {
        let term_c = term.clone();
        let max_results_c = max_results.clone();
        let case_insensitive_c = case_insensitive.clone();
        children.push(thread::spawn(move || {
            match get_file_reader(path.unwrap().path().as_path()) {
                Ok(br) => return do_search(br, term_c, max_results_c, case_insensitive_c),
                Err(e) => bail!(e)
            }
        }));
    }
    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }
    return Ok(());
}

fn search(input_path : &Path, term : String, max_results : usize, case_insensitive : bool,
          partitioned : bool) -> Result<()> {
    if partitioned {
        return search_partitioned(input_path, term, max_results, case_insensitive);
    }
    else {
        match get_file_reader(input_path) {
            Ok(br) => return do_search(br, term, max_results, case_insensitive),
            Err(e) => bail!(e)
        }
    }
}

fn run(term : Option<String>, max_results : usize, case_insensitive : bool,
       partitioned : bool, path : &Path)
    -> Result<()> {
    match term {
       Some(s) => return search(&path, s, max_results, case_insensitive, partitioned),
       None    => return random_entry(&path)
    };
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut default = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    default.push(DEFAULT_DB_SUB_PATH);
    match args.db_path {
       Some(path) => return run(args.search_term, args.max_results,
                                args.case_insensitive, args.partitioned, Path::new(&path)),
       None       => return run(args.search_term, args.max_results,
                                args.case_insensitive, args.partitioned, default.as_path())
    };
}
