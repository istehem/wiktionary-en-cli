fn set_rusc_env_variable(key: &str, value: String) {
    println!("cargo:rustc-env={}={}", key, value);
}

fn main() {
    set_rusc_env_variable(
        "CACHING_DIR",
        format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "cache"),
    );
}
