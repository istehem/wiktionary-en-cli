use anyhow::{bail, Result};

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

    let db = sled::open(&path)
        .map_err(|err| anyhow::Error::new(err).context(format!("cannot open db: {}", path)));

    let json = anyhow_serde::to_string(value)
        .map_err(|err| err.context(format!("cannot serialize entry")));

    return json.and_then(|json| {
        db.and_then(|db| {
            db.insert(term, json.as_bytes())
                .map_err(|err| anyhow::Error::new(err))
                .map(|_| return)
        })
    });
}

pub fn get_cached_db_entry<T: for<'a> serde::Deserialize<'a>>(
    term: &String,
    language: &Language,
) -> Result<Option<T>> {
    let path = DICTIONARY_CACHING_PATH!(language.value());

    let db = sled::open(&path)
        .map_err(|err| anyhow::Error::new(err).context(format!("cannot open db: {}", path)));

    match db.and_then(|db| db.get(term).map_err(|err| anyhow::Error::new(err))) {
        Ok(Some(b)) => {
            return String::from_utf8((&b).to_vec())
                .map_err(|err| anyhow::Error::new(err))
                .and_then(|s| anyhow_serde::from_str(&s))
        }
        Ok(_) => return Ok(None),
        Err(err) => bail!(err),
    };
}
