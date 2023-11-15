# Rust-Chess-Engine

A chess engine in development. I'm learning Rust as I go, so any feedback is appreciated.

Currently, the engine only contains legal move generation. The results so far seem quite promising regarding performance:

perft(1) on startpos: 20 moves in 54 ns. (benchmarked using Criterion)
More complex positions also average 2 to 3 nanoseconds per generated move.
