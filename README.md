# Rust-Chess-Engine

A chess engine in development. I'm learning Rust and engine dev as I go, so any feedback is appreciated.

## Lichess

[The bot runs on Lichess.org!](https://lichess.org/@/LoevBot)
*(Currently running on a 2GHz `Odroid N2+`)*

## Search features:

- iterative deepening
- alpha-beta negamax
- transposition table
    - current replacement scheme:
        1. previous PV always remains
        2. previous iteration gets replaced
        3. depth-preferred replacement
- move ordering:
    1. TT move
    2. MVV-LVA

## Eval features:

- PeSTO eval

# How to build

Since version `0.1.7`, the [uci repository](https://github.com/Loev06/uci) has been combined with this repository, so building from source should now be trivial:

In the root directory, run
`cargo build --release`

## Benchmarks

Some [Criterion](https://crates.io/crates/criterion) benchmarks used for development can be run with
`cargo bench --bench bench`

One statistic which I'm proud of:
> `perft(6)`takes on average`430 ms`, which is`~276 MNPS!`(bulk counting, no hashing)

## Perft test suite

Run a perft test suite with
`cargo test --release -- --nocapture`
This performs perft on 32 tricky positions and compares it with expected perft results.