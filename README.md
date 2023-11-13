# Rust-Chess-Engine

A chess engine in development. I'm learning Rust as I go, so any feedback is appreciated.

Currently, the engine only contains legal move generation of the non-special moves (no castling, promotion or en passant yet). The results so far seem quite promising regarding performance:

perft(1) on startpos: 20 moves in 52 ns.
More complex positions also average 2 to 3 nanoseconds per generated move.
(4.7 GHz CPU, so that would average 4.7E9 * 3E-9 = ~14 clock cycles per move, which seems way too low.. There might be an error in my calculations)
