use std::io::{BufReader};
use anyhow::{Result, bail};
use std::path::{Path};
use std::fs::File;

pub fn get_file_reader(path: &Path) -> Result<BufReader<File>> {
    let file_buffer_result =  File::open(path)
        .map(|f| BufReader::new(f))
        .map_err(|err| anyhow::Error::new(err));
    match file_buffer_result {
        ok@Ok(_) => return ok,
        _        => bail!("No such DB file: '{}'", path.display())

    }
}

