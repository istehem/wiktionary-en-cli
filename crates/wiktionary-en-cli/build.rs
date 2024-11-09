macro_rules! PROJECT_DIR {
    () => {
        format!("{}/../..", env!("CARGO_MANIFEST_DIR"))
    };
}

fn set_rusc_env_variable(key: &str, value: String) {
    println!("cargo:rustc-env={}={}", key, value);
}

fn main() {
    set_rusc_env_variable("CACHING_DIR", format!("{}/{}", PROJECT_DIR!(), "cache"));

    set_rusc_env_variable("DICTIONARY_DIR", format!("{}/{}", PROJECT_DIR!(), "files"));

    set_rusc_env_variable("PROJECT_DIR", String::from(PROJECT_DIR!()));
}
