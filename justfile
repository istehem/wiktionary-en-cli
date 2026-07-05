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
  podman compose -f ./couchdb/docker-compose.yaml up --force-recreate -d
  podman compose -f ./sonic/docker-compose.yaml up --force-recreate -d

[group: 'test']
test-couchdb-client:
  cargo test -p tests --test test-couchdb-client -- --nocapture

[group: 'test']
test-config:
  cargo test -p tests --test test-config -- --nocapture

# install wiktionary-en-import with standard features
[group: 'install']
install-wiktionary-en-import:
  cargo install --path crates/wiktionary-en-import --features sonic

# install wiktionary-en-cli with standard features
[group: 'install']
install-wiktionary-en-cli:
  cargo install --path crates/wiktionary-en-cli --features sonic

# install all binaries with standard features
[group: 'install']
install: install-wiktionary-en-import install-wiktionary-en-cli

# lint using clippy
[group: 'lint']
lint:
  cargo clippy --manifest-path crates/wiktionary-en-cli/Cargo.toml --features sonic
  cargo clippy --manifest-path crates/wiktionary-en-import/Cargo.toml --features sonic

