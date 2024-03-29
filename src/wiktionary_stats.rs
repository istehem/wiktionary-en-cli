use colored::ColoredString;
use colored::Colorize;
use std::io::BufRead;
use std::path::Path;

use utilities::colored_string_utils::*;
use utilities::file_utils::*;

pub struct Stats {
    path: String,
    caching_dir: String,
    number_of_entries: Option<usize>,
    file_size: Option<u64>,
}

impl Stats {
    pub fn to_pretty_string(&self) -> ColoredString {
        let mut res: Vec<ColoredString> = Vec::new();
        res.push(format!("{:<19}: {}", "dictionary file".green(), self.path).normal());
        res.push(format!("{:<19}: {}", "caching dir".green(), self.caching_dir).normal());

        if let Some(n) = self.number_of_entries {
            res.push(
                format!(
                    "{:<19}: {}",
                    "dictionary entries".green(),
                    format_integer(n)
                )
                .normal(),
            );
        }
        if let Some(s) = self.file_size {
            res.push(
                format!(
                    "{:<19}: {} {}",
                    "dictionary size".green(),
                    format_integer(s.try_into().unwrap()),
                    "MB"
                )
                .normal(),
            );
        }
        return NEWLINE.normal().join(res);
    }
}

pub fn calculate_stats(path: &Path) -> Stats {
    return Stats {
        path: path.display().to_string(),
        caching_dir: String::from(env!("CACHING_DIR")),
        file_size: file_size_in_megabytes(path),
        number_of_entries: number_of_entries(path),
    };
}

fn file_size_in_megabytes(input_path: &Path) -> Option<u64> {
    if input_path.is_dir() {
        return None;
    }
    if let Ok(metadata) = input_path.metadata() {
        return Some(metadata.len() / 1024);
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
