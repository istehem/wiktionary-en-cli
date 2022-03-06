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
    max_results : usize
}

fn print_entry(json : &DictionaryEntry) {
    println!("{}", json.to_pretty_string());
}

fn get_file_reader(path: &Path) -> Result<BufReader<File>> {
    let file_buffer_result =  File::open(path).map(|f| BufReader::new(f));
    match file_buffer_result {
        Ok(buffer) => return Ok(buffer),
        _          => bail!("No such DB file: '{}'", path.display().to_string())

    }
}

fn find_entry(file_reader : BufReader<File>, index : usize) -> Option<DictionaryEntry> {
    for (i, line) in file_reader.lines().enumerate() {
        if i == index {
            return line.ok().and_then(|l| wiktionary_data::parse(&l).ok());
        }
    }
    return None;
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
        .and_then(|(br, index)| find_entry(br, index)) {
        Some(json) => print_entry(&json),
        _          => ()
    }
    return Ok(());
}

fn do_search(file_reader : BufReader<File>, term : String, max_results : usize)
    -> Result<()> {
    let mut result : Option<DictionaryEntry> = None;
    let mut full_matches : Vec<DictionaryEntry> = Vec::new();
    let mut min_distance = usize::MAX;
    for line in file_reader.lines() {
        let json : DictionaryEntry = wiktionary_data::parse(&line?)?;
        let distance = edit_distance(&json.word, &term);
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

fn search(input_path : &Path, term : String, max_results : usize) -> Result<()> {
    match get_file_reader(input_path) {
       Ok(br) => return do_search(br, term, max_results),
       Err(e) => bail!(e)
    }
}


fn run(term : Option<String>, max_results : usize, path : &Path) -> Result<()> {
    match term {
       Some(s) => return search(&path, s, max_results),
       None    => return random_entry(&path)
    };
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let mut default = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    default.push(DEFAULT_DB_SUB_PATH);
    match args.db_path {
       Some(path) => return run(args.search_term, args.max_results, Path::new(&path)),
       None       => return run(args.search_term, args.max_results, default.as_path())
    };
}
