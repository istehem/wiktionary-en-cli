use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn get_file_reader(path: &Path) -> Result<BufReader<File>> {
    return File::open(path)
        .map(|f| BufReader::new(f))
        .map_err(|err| anyhow!(err).context(format!("Couldn't open file: '{}'", path.display())));
}
