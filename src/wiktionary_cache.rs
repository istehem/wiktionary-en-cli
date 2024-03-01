use anyhow::{bail, ensure, Result};

use utilities::anyhow_serde;
use utilities::language::*;

macro_rules! DICTIONARY_CACHING_PATH {
    ($language:expr) => {
        format!("{}/wiktionary-cache-{}", env!("CACHING_DIR"), $language)
    };
}

pub fn write_db_entry_to_cache<T: serde::Serialize>(
    term: &String,
    value: &T,
    language: &Language,
) -> Result<()> {
    // this directory will be created if it does not exist
    let path = DICTIONARY_CACHING_PATH!(language.value());

    let db = sled::open(&path);
    ensure!(db.is_ok(), format!("error writing to cache db: {}", path));
    let json_string = anyhow_serde::to_string(value);
    ensure!(json_string.is_ok(), "cannot serialize entry");

    return db
        .unwrap()
        .insert(term, json_string.unwrap().as_bytes())
        .map_err(|err| anyhow::Error::new(err))
        .map(|_| return);
}

pub fn get_cached_db_entry<T: for<'a> serde::Deserialize<'a>>(
    term: &String,
    language: &Language,
) -> Result<T> {
    let path = DICTIONARY_CACHING_PATH!(language.value());

    let db = sled::open(&path);
    ensure!(db.is_ok(), format!("error reading from cache db: {}", path));

    match db.unwrap().get(term) {
        Ok(Some(b)) => {
            return String::from_utf8((&b).to_vec())
                .map_err(|err| anyhow::Error::new(err))
                .and_then(|s| anyhow_serde::from_str(&s))
        }
        Ok(_) => bail!("entry not found"),
        Err(err) => bail!(err),
    };
}
