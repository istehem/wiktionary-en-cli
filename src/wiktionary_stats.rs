use std::path::{Path};
use std::io::BufRead;
use colored::ColoredString;

mod file_reader;
use crate::get_file_reader;

pub struct Stats {
    path: String,
    number_of_entries: Option<usize>,
    file_size: Option<u64>
}

pub trait Join {
    fn join(&self, list : Vec<Self>) -> Self where Self: Sized;
}

impl Join for String {
    fn join(&self, list : Vec<String>) -> String {
        let mut res : String = String::new();
        let len : usize = list.len();
        for (i, string) in list.iter().enumerate() {
            res = format!("{}{}", res, string);
            if i < len - 1 {
                res = format!("{}{}", res, self);
            }
        }
        return res.clone();
    }
}

impl Stats {
    pub fn to_pretty_string(&self) -> String {
        let mut res : Vec<String> = Vec::new();
        res.push(format!("{:<19}: {}","dbfile", self.path));

        if let Some(n) = self.number_of_entries {
            res.push(format!("{:<19}: {}", "dictionary entries", format_integer(n)));
        }
        if let Some(s) = self.file_size {
            res.push(
                format!("{:<19}: {} {}", "size",
                    format_integer(s.try_into().unwrap()), "MB"));

        }
        return "\n".to_string().join(res);
    }
}

pub fn calculate_stats(path: &Path) -> Stats {
    return Stats {
        path: path.display().to_string(),
        file_size: file_size_in_megabytes(path),
        number_of_entries: number_of_entries(path),
    }
}

fn file_size_in_megabytes(input_path : &Path) -> Option<u64> {
    if input_path.is_dir() {
        return None;
    }
    if let Ok(metadata) = input_path.metadata() {
        return Some(metadata.len() / 1024);
    }
    return None;
}

fn number_of_entries(input_path : &Path) -> Option<usize> {
    if input_path.is_dir(){
        return None;
    }
    if let Ok(br) = get_file_reader(input_path) {
        return Some(br.lines().count());
    }
    return None;
}

fn format_integer(number: usize) -> String {
    return number.to_string()
                 .as_bytes()
                 .rchunks(3)
                 .rev()
                 .map(std::str::from_utf8)
                 .collect::<Result<Vec<&str>, _>>()
                 .unwrap()
                 .join(",");
}


