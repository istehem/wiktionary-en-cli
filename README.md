# Installation
The project depends on [PoloDB](https://github.com/PoloDB/PoloDB) that
in turn depends on [https://github.com/rust-rocksdb/rust-rocksdb](https://github.com/rust-rocksdb/rust-rocksdb),
which support Rust bindings for [RocksDB](https://rocksdb.org/).

For Cargo to build the dependencies correctly the Clang compiler and LLVM needs to be installed.
Also, the development dependencies for OpenSSL need to be installed.

On Debian the requirements can be install with:

```console
sudo apt install libclang-dev libssl-dev
```

The easiest way the install wiktionary-en-cli is to use cargo-make
You can install it with:
```console
cargo install cargo-make
```

Once cargo-make is installed, install wiktionary-en-cli with:
```console
cargo make install
```

## Sonic Feature
The sonic feature is enabled by default and requires one to have a [sonic server](https://github.com/valeriansaliou/sonic) running.

The sonic server can be started in podman or docker.
```console
cd "$(git rev-parse --show-toplevel)/sonic" && podman compose up -d
```

You can also turn this feature off by installing wiktionary-en-cli with:
```console
cargo make --env USE_SONIC_FEATURE=false install
```

