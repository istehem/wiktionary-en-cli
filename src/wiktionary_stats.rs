use colored::ColoredString;
use colored::Colorize;
use std::io::BufRead;
use std::path::Path;

use utilities::cache_utils;
use utilities::colored_string_utils::*;
use utilities::file_utils::*;

macro_rules! format_key_value {
    ($key:expr, $value:expr) => {
        format!("{:<19}: {}", $key, $value)
    };
}

const MEGABYTE: u64 = 1024*1024;

pub struct Stats {
    path: String,
    caching_dir: String,
    cache_size: Option<u64>,
    cached_entries: Option<usize>,
    number_of_entries: Option<usize>,
    file_size: Option<u64>,
}

impl Stats {
    pub fn to_pretty_string(&self) -> ColoredString {
        let mut res: Vec<ColoredString> = Vec::new();
        res.push(format_key_value!("dictionary file".green(), self.path).normal());
        res.push(format_key_value!("caching dir".green(), self.caching_dir).normal());

        if let Some(n) = self.cached_entries {
            res.push(format_key_value!("cached entries".green(), format_integer(n)).normal());
        }
        if let Some(n) = self.cache_size {
            res.push(
                format_key_value!(
                    "cache size".green(),
                    format!("{} {}", format_integer(n.try_into().unwrap()), "M")
                )
                .normal(),
            );
        }
        if let Some(n) = self.number_of_entries {
            res.push(format_key_value!("dictionary entries".green(), format_integer(n)).normal());
        }
        if let Some(s) = self.file_size {
            res.push(
                format_key_value!(
                    "dictionary size".green(),
                    format!("{} {}", format_integer(s.try_into().unwrap()), "M")
                )
                .normal(),
            );
        }
        return NEWLINE.normal().join(res);
    }
}

pub fn calculate_stats(dictionary_path: &Path, cache_path: &String) -> Stats {
    return Stats {
        path: dictionary_path.display().to_string(),
        caching_dir: cache_path.clone(),
        cache_size: cache_size_in_megabytes(cache_path),
        cached_entries: cached_entries(cache_path),
        file_size: file_size_in_megabytes(dictionary_path),
        number_of_entries: number_of_entries(dictionary_path),
    };
}

fn cached_entries(cache_path: &String) -> Option<usize> {
    if let Ok(number) = cache_utils::get_number_of_entries(cache_path) {
        return Some(number);
    }
    return None;
}

fn cache_size_in_megabytes(cache_path: &String) -> Option<u64> {
    if let Ok(number) = cache_utils::get_size_on_disk(cache_path) {
        return Some(number / MEGABYTE);
    }
    return None;
}

fn file_size_in_megabytes(input_path: &Path) -> Option<u64> {
    if input_path.is_dir() {
        return None;
    }
    if let Ok(metadata) = input_path.metadata() {
        return Some(metadata.len() / MEGABYTE);
    }
    return None;
}

fn number_of_entries(input_path: &Path) -> Option<usize> {
    if input_path.is_dir() {
        return None;
    }
    if let Ok(br) = get_file_reader(input_path) {
        return Some(br.lines().count());
    }
    return None;
}
