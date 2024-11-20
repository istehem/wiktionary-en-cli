#[macro_export]
macro_rules! PROJECT_DIR {
    () => {
        format!("{}/../..", env!("CARGO_MANIFEST_DIR"))
    };
}

#[macro_export]
macro_rules! DICTIONARY_DIR {
    () => {
        format!("{}/{}", PROJECT_DIR!(), "files")
    };
}

#[macro_export]
macro_rules! CACHING_DIR {
    () => {
        format!("{}/{}", PROJECT_DIR!(), "cache")
    };
}

#[macro_export]
macro_rules! DICTIONARY_CACHING_PATH {
    ($language:expr) => {
        format!("{}/wiktionary-cache-{}", CACHING_DIR!(), $language)
    };
}

#[macro_export]
macro_rules! DICTIONARY_DB_PATH {
    ($language:expr) => {
        format!("{}/wiktionary-{}.json", DICTIONARY_DIR!(), $language)
    };
}

#[macro_export]
macro_rules! DEFAULT_DB_PARTITIONED_DIR {
    () => {
        format!("{}/partitioned", DICTIONARY_DIR!())
    };
}

#[macro_export]
macro_rules! DICTIONARY_POLO_DB_DIR {
    () => {
        format!("{}/wiktionary-polo-db", PROJECT_DIR!())
    };
}
