use anyhow::{bail, Result};

use utilities::language::*;

macro_rules! DICTIONARY_CACHING_PATH {
    ($language:expr) => {
        format!(
            "{}/cache/wiktionary-cache-{}",
            env!("CACHING_DIR"),
            $language
        )
    };
}

pub fn write_db_entry_to_cache(term: &String, value: &String, language: &Language) -> Result<()> {
    // this directory will be created if it does not exist
    let path = DICTIONARY_CACHING_PATH!(language.value());

    let db = sled::open(path)?;
    let key = term;

    return db
        .insert(key, value.as_bytes())
        .map_err(|err| anyhow::Error::new(err))
        .map(|_| return);
}

pub fn get_cached_db_entry<T: for<'a> serde::Deserialize<'a>>(
    term: &String,
    language: &Language,
) -> Result<T> {
    let path = DICTIONARY_CACHING_PATH!(language.value());

    let db = sled::open(path)?;

    match db.get(term) {
        Ok(Some(b)) => {
            return String::from_utf8((&b).to_vec())
                .map_err(|err| anyhow::Error::new(err))
                .and_then(|s| parse(&s))
        }
        Ok(_) => bail!("entry not found"),
        Err(err) => bail!(err),
    };
}

pub fn parse<T: for<'a> serde::Deserialize<'a>>(line: &String) -> Result<T> {
    return serde_json::from_str(line).map_err(|err| anyhow::Error::new(err));
}
