# Rust-Chess-Engine

A chess engine in development. I'm learning Rust as I go, so any feedback is appreciated.

Search features:
- alpha-beta negamax
- move ordering:
    - MVV-LVA

Eval features:
- Piece values

# UCI

The engine is UCI compatible! See https://github.com/Loev06/uci for the source code.

# How to build
This repository only contains the library for the chess engine, so building it is not (yet) convenient from GitHub. I will improve the git structure some time later, but here is a rudimentary method for builing:

1. Put both this repository (named `bot` below) and the corresponding UCI repository (named `uci` below) as childs in a parent directory, and reference them with a new `Cargo.toml`:

```toml
# Cargo.toml

[workspace]

members = [
    "chess_engine",
    "uci",
    "bot"
]

resolver = "2"
```

2. Now we need to make a binary to run the uci program. Make a directory `/chess_engine/`, and add `/chess_engine/Cargo.toml` and `/chess_engine/src/main.rs` with the following contents:

```toml
# /chess_engine/Cargo.toml

[package]
name = "chess_engine"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
lichess = { path = "../uci"}
```

```rust
// /chess_engine/src/main.rs

use uci::Uci;

fn main() {
    Uci::new().run();
}
```
3. Run `cargo build --release`
