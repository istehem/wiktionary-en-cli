#[macro_export]
macro_rules! DICTIONARY_CACHING_PATH {
    ($language:expr) => {
        format!("{}/wiktionary-cache-{}", env!("CACHING_DIR"), $language)
    };
}

#[macro_export]
macro_rules! DICTIONARY_DB_PATH {
    ($language:expr) => {
        format!("{}/wiktionary-{}.json", env!("DICTIONARY_DIR"), $language)
    };
}

#[macro_export]
macro_rules! DEFAULT_DB_PARTITIONED_DIR {
    () => {
        format!("{}/partitioned", env!("DICTIONARY_DIR"))
    };
}
