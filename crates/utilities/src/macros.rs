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
macro_rules! DEFAULT_PARTIONED_DB_DIR_PATH {
    () => {
        env!("DEFAULT_PARTIONED_DB_DIR_PATH")
    };
}

#[macro_export]
macro_rules! DICTIONARY_POLO_DB_DIR {
    () => {
        env!("DICTIONARY_POLD_DB_DIR")
    };
}
