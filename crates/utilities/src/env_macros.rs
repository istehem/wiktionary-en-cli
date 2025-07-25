#[macro_export]
macro_rules! DICTIONARY_DB_PATH {
    ($language:expr) => {
        format!(env!("DICTIONARY_DB_PATH_PLACEHOLDER"), $language)
    };
}

#[macro_export]
macro_rules! DICTIONARY_POLO_DB_DIR {
    () => {
        env!("DICTIONARY_POLD_DB_DIR")
    };
}

#[macro_export]
macro_rules! DICTIONARY_CONFIG {
    () => {
        env!("DICTIONARY_CONFIG")
    };
}

#[macro_export]
macro_rules! DICTIONARY_EXTENSIONS {
    () => {
        env!("DICTIONARY_EXTENSIONS")
    };
}

#[macro_export]
macro_rules! LUA_DIR {
    () => {
        env!("LUA_DIR")
    };
}
