# Installation
The easiest way the install wiktionary-en-cli is to use cargo-make. 
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
You can turn this feature of by installing wiktionary-en-cli with:
```console
cargo make --env USE_SONIC_FEATURE=false install
```

