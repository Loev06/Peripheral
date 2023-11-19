use crate::chess::precomputed;

use super::{Square, Bitboard, PieceType::{self, *}, Color::*};

pub const fn get_piece_name(pt: Option<PieceType>) -> char {
    match pt {
        Some(pt) => match pt {
            Pawn(White)     => 'P',
            Knight(White)   => 'N',
            Bishop(White)   => 'B',
            Rook(White)     => 'R',
            Queen(White)    => 'Q',
            King(White)     => 'K',
            Pawn(Black)     => 'p',
            Knight(Black)   => 'n',
            Bishop(Black)   => 'b',
            Rook(Black)     => 'r',
            Queen(Black)    => 'q',
            King(Black)     => 'k',
            _ => panic!("Not a valid piece")
        }
        None => '.'
    }
}

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

