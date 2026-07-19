# Installation

The easiest way to install wiktionary-en-cli is to use the just command runner:
```console
just install
```

For this to work properly some environment variables need to be set in the environment file.
```console
touch wiktionary-en.env
```

Then, in your favorite editor, add the following to this environment file:
```env
COUCH_DB_PASSWORD="<your-couchdb-password>"
SONIC_PASSWORD="<your-sonic-password>"
```
```
```
## Infrastructure

You can start the required infrastructure in podman by executing:
```console
just start-background-services
```

Alternative you can start the services individually.

### CouchDB
Dictionary data is stored in CouchDB. You can start it with:
```console
cd "$(git rev-parse --show-toplevel)/couchdb" && podman compose up -d
```

### Sonic Feature
The sonic feature is enabled by default and requires one to have a [sonic server](https://github.com/valeriansaliou/sonic) running.

The sonic server can be started by executing:
```console
cd "$(git rev-parse --show-toplevel)/sonic" && podman compose up -d
```

You can also turn this feature off by installing wiktionary-en-cli with:
```console
just install-no-sonic
```

