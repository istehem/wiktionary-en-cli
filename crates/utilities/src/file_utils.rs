use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;

use crate::language::Language;

use crate::DICTIONARY_DB_PATH;

pub fn get_file_reader(path: &Path) -> Result<BufReader<File>> {
    File::open(path)
        .map(BufReader::new)
        .map_err(|err| anyhow!(err).context(format!("Couldn't open file: '{}'", path.display())))
}

pub fn get_db_path(path: Option<String>, language: &Language) -> PathBuf {
    if let Some(path) = path {
        return PathBuf::from(path);
    }
    PathBuf::from(DICTIONARY_DB_PATH!(language))
}
