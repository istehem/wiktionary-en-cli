# start services needed to run wiktionary-cli
[group: 'setup']
start-background-services:
  podman compose -f ./couchdb/docker-compose.yaml up
