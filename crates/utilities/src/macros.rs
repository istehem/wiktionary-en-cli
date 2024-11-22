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
macro_rules! DICTIONARY_CACHING_PATH {
    ($language:expr) => {
        format!(env!("DICTIONARY_CACHING_PATH_PLACEHOLDER"), $language)
    };
}

#[macro_export]
macro_rules! DICTIONARY_DB_PATH {
    ($language:expr) => {
        format!(env!("DICTIONARY_DB_PATH_PLACEHOLDER"), $language)
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
        env!("DICTIONARY_POLD_DB_DIR")
    };
}
