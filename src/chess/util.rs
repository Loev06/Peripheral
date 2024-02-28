use crate::chess::precomputed;

use super::{Bitboard, Color, Square};

pub fn print_bb(bb: Bitboard) {
    println!(
        "{}",
        (0..8).rev().map(|y| {
            (0..8).map(|x| {
                if bb & bitboard_from_square(square_from_coord(x, y)) == precomputed::EMPTY {"."} else {"X"}
            })
            .fold(String::new(), |a, b| a + &b)
            .to_owned()
        })
        .fold(String::new(), |a, b| a + &b + "\n")
    );
}

#[inline(always)]
pub const fn shift_dir(bb: Bitboard, shift: u8, dir: Color) -> Bitboard {
    match dir {
        Color::White => bb << shift,
        Color::Black => bb >> shift
    }
}

#[inline(always)]
pub const fn square_from_coord(x: usize, y: usize) -> Square {
    debug_assert!((x + (y << 3)) < 64);
    (x + (y << 3)) as Square
}

#[inline(always)]
pub const fn ls1b_from_bitboard(bb: Bitboard) -> Square {
    bb.trailing_zeros() as Square
}

#[inline(always)]
pub fn pop_ls1b(bb: &mut Bitboard) -> Square {
    debug_assert!(*bb != precomputed::EMPTY);
    let sq = ls1b_from_bitboard(*bb);
    *bb ^= bitboard_from_square(sq);
    sq
}

#[inline(always)]
pub fn pop_ls1b_as_bb(bb: &mut Bitboard) -> Bitboard {
    let out = bitboard_from_square(ls1b_from_bitboard(*bb));
    *bb ^= out;
    out
}

#[inline(always)]
pub const fn bitboard_from_square(square: Square) -> Bitboard {
    debug_assert!(square < 64);
    1u64 << square
}

#[inline(always)]
pub const fn get_square_x(square: Square) -> u8 {
    debug_assert!(square < 64);
    square & 0b000111
}

#[inline(always)]
pub const fn get_square_y(square: Square) -> u8 {
    debug_assert!(square < 64);
    square >> 3
}

#[inline(always)]
pub const fn is_out_of_bounds(x: isize, y: isize) -> bool {
    x < 0 || x >= 8 || y < 0 || y >= 8
}

pub fn piece_name_from_usize(pt: usize) -> char {
    *"PNBRQKpnbrqk".as_bytes().get(pt).expect(&format!("Not a valid piece: {}", pt)) as char
}