use anyhow::{bail, Result};
use colored::ColoredString;
use colored::Colorize;
use std::fmt;
use std::io::BufRead;
use std::path::Path;

use utilities::colored_string_utils::*;
use utilities::file_utils::*;
use utilities::language::Language;
use wiktionary_en_db::client::WiktionaryDbClient;

use utilities::DICTIONARY_POLO_DB_DIR;

macro_rules! format_key_value {
    ($key:expr, $value:expr) => {
        format!("{:<19}: {}", $key.green(), $value).normal()
    };
}

macro_rules! format_float {
    ($value:expr) => {
        format!("{:.2}", $value).yellow()
    };
}

const MEGABYTE: f64 = 1024.0 * 1024.0;

pub struct Stats {
    path: String,
    database_dir: String,
    database_entries: usize,
    number_of_entries: usize,
    file_size: Option<f64>,
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.to_pretty_string())
    }
}

impl Stats {
    fn to_pretty_string(&self) -> ColoredString {
        let mut res: Vec<ColoredString> = Vec::new();
        res.push(format_key_value!("dictionary file", self.path));
        res.push(format_key_value!("database dir", self.database_dir));

        res.push(format_key_value!(
            "dabase entries",
            format_integer(self.database_entries)
        ));
        res.push(format_key_value!(
            "dictionary entries",
            format_integer(self.number_of_entries)
        ));
        if let Some(s) = self.file_size {
            res.push(format_key_value!(
                "dictionary size",
                format!("{} {}", format_float!(s), "M")
            ));
        }
        NEWLINE.normal().join(res)
    }

    pub fn calculate_stats(dictionary_path: &Path, language: &Language) -> Result<Stats> {
        calculate_stats(dictionary_path, language)
    }
}

fn calculate_stats(dictionary_path: &Path, language: &Language) -> Result<Stats> {
    let client = WiktionaryDbClient::init(*language)?;
    Ok(Stats {
        path: dictionary_path.display().to_string(),
        database_dir: String::from(DICTIONARY_POLO_DB_DIR!()),
        database_entries: client.number_of_entries()? as usize,
        file_size: file_size_in_megabytes(dictionary_path),
        number_of_entries: number_of_entries(dictionary_path)?,
    })
}

fn file_size_in_megabytes(input_path: &Path) -> Option<f64> {
    if input_path.is_dir() {
        return None;
    }
    if let Ok(metadata) = input_path.metadata() {
        return Some(metadata.len() as f64 / MEGABYTE);
    }
    None
}

fn number_of_entries(input_path: &Path) -> Result<usize> {
    if input_path.is_dir() {
        bail!("path {} is a directory", input_path.display());
    }
    Ok(get_file_reader(input_path)?.lines().count())
}
