# Ludum Dare 52

This is a placeholder repository for Ludum Dare 52.

[![Rust Checks](https://github.com/matanlurey/ld52/actions/workflows/rust.yml/badge.svg)](https://github.com/matanlurey/ld52/actions/workflows/rust.yml)

## How to play

TBD

## Developing

Only Rust (edition 2021) is required to develop and build from source.

Clone this repository and then run:

```bash
cargo run
```

### Web Assembly

By default, the game is built with OpenGL. To run on the web, it uses [WASM][].

[wasm]: https://webassembly.org/

First, install [`cargo-make`](https://github.com/sagiegurari/cargo-make):

```bash
cargo install --force cargo-make
```

Next, install all other required dependencies:

```bash
cargo make install
```

Now you can build the game to Web Assembly:

```bash
# Builds the game as WASM binaries to /web.
cargo make build-wasm
```

To run the game in a web browser (locally), run:

```bash
# Opens a web browser to http://0.0.0.0:8000, serving the /web directory.
cargo make serve-wasm
```
