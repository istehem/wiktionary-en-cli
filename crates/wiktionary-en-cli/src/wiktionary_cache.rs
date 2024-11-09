use anyhow::Result;

use utilities::cache_utils;
use utilities::language::*;

pub fn write_db_entry_to_cache<T: serde::Serialize>(
    term: &String,
    value: &T,
    language: &Language,
) -> Result<()> {
    let path = utilities::DICTIONARY_CACHING_PATH!(language.value());
    return cache_utils::write_db_entry_to_cache(&path, term, value);
}

pub fn get_cached_db_entry<T: for<'a> serde::Deserialize<'a>>(
    term: &String,
    language: &Language,
) -> Result<Option<T>> {
    let path = utilities::DICTIONARY_CACHING_PATH!(language.value());
    return cache_utils::get_cached_db_entry(&path, term);
}
