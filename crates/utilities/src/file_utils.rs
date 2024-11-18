use anyhow::{bail, Result};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn get_file_reader(path: &Path) -> Result<BufReader<File>> {
    let file_buffer_result = File::open(path)
        .map(|f| BufReader::new(f))
        .map_err(|err| anyhow::Error::new(err));
    match file_buffer_result {
        ok @ Ok(_) => return ok,
        Err(err) => bail!(err.context(format!("Coldn't open DB file: '{}'", path.display()))),
    }
}
