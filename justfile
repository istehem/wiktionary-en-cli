PROJECT_ROOT := justfile_directory()
set export := true
DICTIONARY_DB_PATH_PLACEHOLDER := PROJECT_ROOT + "/wiktionary-en-json-extract/wiktionary-{}.jsonl"
DICTIONARY_POLD_DB_DIR := PROJECT_ROOT + "/wiktionary-en-polo-db/"
DICTIONARY_CONFIG := PROJECT_ROOT + "/wiktionary-en-config.lua"
DICTIONARY_EXTENSIONS := PROJECT_ROOT + "/wiktionary-en-extensions.lua"
LUA_DIR := PROJECT_ROOT + "/lua"

# start services needed to run wiktionary-cli
[group: 'setup']
start-background-services:
  podman compose -f ./couchdb/docker-compose.yaml up --force-recreate


[group: 'test']
test-couchdb-client:
  cargo test -p tests --test test-couchdb-client -- --nocapture

