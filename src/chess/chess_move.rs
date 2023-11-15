use std::fmt;
use bitflags::bitflags;

use super::{Square, precomputed};

const SPECIAL_MOVE_NAMES: [&str; 16] = [
    "",
    ": double pawn push",
    ": king castle",
    ": queen castle",
    ": X",
    ": X en-passant",
    ": INVALID MOVE FLAG: 0110",
    ": INVALID MOVE FLAG: 0111",
    ": =N",
    ": =B",
    ": =R",
    ": =Q",
    ": X =N",
    ": X =B",
    ": X =R",
    ": X =Q",
];

// https://www.chessprogramming.org/Encoding_Moves#From-To_Based
bitflags! {
    #[derive(Clone, Copy)]
    pub struct Move: u16 {
        const FROM      = 0b0000_000000_111111;
        const TO        = 0b0000_111111_000000;
        const SPECIAL0  = 0b0001_000000_000000;
        const SPECIAL1  = 0b0010_000000_000000;
        const CAPTURE   = 0b0100_000000_000000;
        const PROMOTION = 0b1000_000000_000000;

        const SPECIAL_BITS = Self::SPECIAL0.bits()
                           | Self::SPECIAL1.bits()
                           | Self::CAPTURE.bits()
                           | Self::PROMOTION.bits();

        const DOUBLE_PAWN_PUSH = Self::SPECIAL0.bits();
        const KING_CASTLE = Self::SPECIAL1.bits();
        const QUEEN_CASTLE = Self::SPECIAL0.bits() | Self::SPECIAL1.bits();
        const EP_CAPTURE = Self::CAPTURE.bits() | Self::SPECIAL0.bits();

        const QUEEN_PROMOTION = Self::PROMOTION.bits() | Self::SPECIAL0.bits() | Self::SPECIAL1.bits();
        const QUEEN_TO_KNIGHT = Self::SPECIAL0.bits() | Self::SPECIAL1.bits();
        const KNIGHT_TO_ROOK = Self::SPECIAL1.bits();
        const ROOK_TO_BISHOP = Self::SPECIAL0.bits() | Self::SPECIAL1.bits();
    }
}

impl Move {
    pub fn new(from: Square, to: Square, special_bits: &Move) -> Move {
        Self((special_bits.bits() | (to as u16) << 6 | (from as u16)).into())
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}{}{}",
            precomputed::SQUARE_NAMES[self.intersection(Move::FROM).bits() as usize],
            precomputed::SQUARE_NAMES[self.intersection(Move::TO).bits() as usize >> 6],
            SPECIAL_MOVE_NAMES[self.intersection(Move::SPECIAL_BITS).bits() as usize >> 12])
        )
    }
}