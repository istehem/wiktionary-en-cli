fn main() {
    println!(
        "cargo:rustc-env=CACHING_DIR={}/{}",
        env!("CARGO_MANIFEST_DIR"),
        "cache"
    )
}
