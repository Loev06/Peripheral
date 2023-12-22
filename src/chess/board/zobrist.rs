// https://github.com/official-stockfish/Stockfish/blob/36db936e769a2e7a95fc4032eec3b79251bbaef5/src/position.cpp#L119

pub const NUM_ZOBRIST_VALUES: usize = 12 * 64 + 8 + 16 + 1;

pub const ZOBRIST_PIECE_SQUARE: [[u64; 64]; 12] = precompute_piece_square().0;
pub const ZOBRIST_EP_SQUARE: [u64; 65] = precompute_ep_square().0;
pub const ZOBRIST_CASTLING: [u64; 16] = precompute_castling().0;
pub const ZOBRIST_BLACK_TO_MOVE: u64 = precompute_black_to_move().0;

const fn rand(seed: u64) -> (u64, u64) {
    let mut seed = seed;
    seed ^= seed >> 12;
    seed ^= seed << 25;
    seed ^= seed >> 27;
    (seed.overflowing_mul(2685821657736338717u64).0, seed)
}

const fn precompute_piece_square() -> ([[u64; 64]; 12], u64) {
    let mut seed = 1070372;
    let mut vals = [[0; 64]; 12];

    let mut pt = 0;
    while pt < 12 {
        let mut sq = 0;
        while sq < 64 {
            (vals[pt][sq], seed) = rand(seed);
            sq += 1;
        }
        pt += 1;
    }

    (vals, seed)
}

const fn precompute_ep_square() -> ([u64; 65], u64) {
    let mut seed = precompute_piece_square().1;
    let mut vals = [0; 65];

    let mut file = 0;
    while file < 8 {
        let num;
        (num, seed) = rand(seed);

        let mut rank = 0;
        while rank < 8 {
            vals[file + 8 * rank] = num;
            rank += 1;
        }
        file += 1;
    }

    (vals, seed)
}

const fn precompute_castling() -> ([u64; 16], u64) {
    let mut seed = precompute_ep_square().1;
    let mut vals = [0; 16];

    let mut cr = 0;
    while cr < 16 {
        (vals[cr], seed) = rand(seed);
        cr += 1;
    }

    (vals, seed)
}

const fn precompute_black_to_move() -> (u64, u64) {
    let seed = precompute_ep_square().1;
    rand(seed)
}