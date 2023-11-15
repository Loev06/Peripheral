use crate::chess::precomputed;

use super::{Square, Bitboard};

pub const fn square_from_coord(x: usize, y: usize) -> Square {
    debug_assert!((x + (y << 3)) < 64);
    (x + (y << 3)) as Square
}

pub const fn ls1b_from_bitboard(bb: Bitboard) -> Square {
    bb.trailing_zeros() as Square
}

pub fn pop_ls1b(bb: &mut Bitboard) -> Square {
    debug_assert!(*bb != precomputed::EMPTY);
    let sq = ls1b_from_bitboard(*bb);
    *bb ^= bitboard_from_square(sq);
    sq
}

pub fn pop_ls1b_as_bb(bb: &mut Bitboard) -> Bitboard {
    let out = bitboard_from_square(ls1b_from_bitboard(*bb));
    *bb ^= out;
    out
}

pub const fn bitboard_from_square(square: Square) -> Bitboard {
    debug_assert!(square < 64);
    1u64 << square
}

pub const fn get_square_x(square: Square) -> u8 {
    debug_assert!(square < 64);
    square & 0b000111
}

pub const fn get_square_y(square: Square) -> u8 {
    debug_assert!(square < 64);
    square >> 3
}

pub const fn is_out_of_bounds(x: isize, y: isize) -> bool {
    x < 0 || x >= 8 || y < 0 || y >= 8
}

