# Peripheral

A chess engine in development. I'm learning Rust and engine dev as I go, so any feedback is appreciated.

## Lichess

[The bot runs on Lichess.org!](https://lichess.org/@/LoevBot)
*(Currently running on a 2GHz `Odroid N2+`)*

## Rating estimations
Rating estimate of version `1.1.3`:

`~2055` CCRL blitz (`+95` against `1.0.0`)

`~2069` Lichess (all time controls). Probably not representative outside of Lichess, since the bot plays on relatively weak hardware, potentially against cherry-picking bots, etc. etc.

SPRT results and ELO calculations can be found [here](https://github.com/Loev06/Rust-Chess-Engine/blob/main/CCRL%20Rating%20Estimate.txt).

## Search features:

- iterative deepening
- alpha-beta negamax
- extended null-move reductions
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

## UCI commands
- **default UCI commands** (not 100% complete, but sufficient for compatibility with e.g. Cute Chess)
- `help`        Show list of known commands
- `d`           Print current board
- `eval`        Static eval of position
- `run`         Run main function of the bot (multipurpose debug command, runs run_bot() located in lib.rs)
- `make [move]` Make move (*e.g. `e2e4`*)
- `undo`        Undo last move made
- `probe`       Probe current position in the transposition table
- `gen`         Get the TT generation of the last search
- `hist`        Print the history of stored keys for threefold detection
- `quit`        Quit

## Benchmarks

Some [Criterion](https://crates.io/crates/criterion) benchmarks used for development can be run with
`cargo bench --bench bench`

One statistic which I'm proud of:
> `perft(6)`takes on average`430 ms`, which is`~276 MNPS!`(bulk counting, no hashing)
> 
> (No-bulk: `2372 ms`, `50 MNPS`)

## Perft test suite

Run a perft test suite with
`cargo test --release -- --nocapture`
This performs perft on 32 tricky positions and compares it with expected perft results.
