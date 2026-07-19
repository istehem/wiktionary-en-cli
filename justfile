PROJECT_ROOT := justfile_directory()
set export := true
DICTIONARY_DB_PATH_PLACEHOLDER := PROJECT_ROOT + "/wiktionary-en-json-extract/wiktionary-{}.jsonl"
DICTIONARY_POLD_DB_DIR := PROJECT_ROOT + "/wiktionary-en-polo-db/"
DICTIONARY_CONFIG := PROJECT_ROOT + "/wiktionary-en-config.lua"
DICTIONARY_EXTENSIONS := PROJECT_ROOT + "/wiktionary-en-extensions.lua"
LUA_DIR := PROJECT_ROOT + "/lua"
COUCH_DB_USER := "admin"
COUCH_DB_HOST := "http://localhost:5984"
SONIC_HOST := "localhost:1491"

set dotenv-required
set dotenv-path := "wiktionary-en.env"

# start services needed to run wiktionary-en-cli
[group: 'setup']
start-background-services:
  podman compose -f {{PROJECT_ROOT}}/couchdb/docker-compose.yaml up --force-recreate -d
  podman compose -f {{PROJECT_ROOT}}/sonic/docker-compose.yaml up --force-recreate -d

# check for outdated dependencies
[group: 'maintenance']
outdated:
  # run `cargo upgrade --dry-run` to check versions defined in [workspace.dependencies]
  cargo outdated -w --root-deps-only

# check for unused dependencies
[group: 'maintenance']
unused:
  cargo machete

[group: 'test']
test-couchdb-client:
  cargo test -p tests --test test-couchdb-client -- --nocapture

[group: 'test']
test-config:
  cargo test -p tests --test test-config -- --nocapture

[group: 'test']
test-download:
  cargo test -p tests --test test-download --features="test-download" -- --nocapture

[group: 'test']
test-extensions:
  cargo test -p tests --test test-extensions -- --nocapture

[group: 'test']
test:
  cargo test -p tests -- --nocapture

# install wiktionary-en-import without the sonic feature
[group: 'install']
install-wiktionary-en-import-no-sonic:
  cargo install --path crates/wiktionary-en-import

# install wiktionary-en-import with standard features
[group: 'install']
install-wiktionary-en-import:
  cargo install --path crates/wiktionary-en-import --features sonic

# install wiktionary-en-cli without the sonic sonic feature
[group: 'install']
install-wiktionary-en-cli-no-sonic:
  cargo install --path crates/wiktionary-en-cli

# install wiktionary-en-cli with standard features
[group: 'install']
install-wiktionary-en-cli:
  cargo install --path crates/wiktionary-en-cli --features sonic

# install all binaries with standard features
[group: 'install']
install: install-wiktionary-en-import install-wiktionary-en-cli

# install all binaries without the standard feature
[group: 'install']
install-no-sonic: install-wiktionary-en-import-no-sonic install-wiktionary-en-cli-no-sonic


# lint using clippy
[group: 'lint']
lint:
  cargo clippy --manifest-path crates/wiktionary-en-cli/Cargo.toml --features sonic
  cargo clippy --manifest-path crates/wiktionary-en-import/Cargo.toml --features sonic
  cargo clippy --manifest-path tests/Cargo.toml

