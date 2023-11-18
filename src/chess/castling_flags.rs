use bitflags::bitflags;

use super::{Square, precomputed};

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct CastlingFlags: u8 {
        const WK = 0b0000_0001;
        const WQ = 0b0000_0010;
        const BK = 0b0000_0100;
        const BQ = 0b0000_1000;

        const WHITE = Self::WK.bits() | Self::WQ.bits();
        const BLACK = Self::BK.bits() | Self::BQ.bits();
        const ALL   = Self::WHITE.bits() | Self::BLACK.bits();
    }
}

impl CastlingFlags {
    pub fn new(data: u8) -> CastlingFlags{
        Self(data.into())
    }

    pub fn update(&mut self, from: Square, to: Square) {
        *self = *self & CASTLING_PER_SQUARE[from as usize] & CASTLING_PER_SQUARE[to as usize];
    }
}

const CASTLING_PER_SQUARE: [CastlingFlags; 64] = precompute_castling_per_square();

const fn precompute_castling_per_square() -> [CastlingFlags; 64] {
    let mut perms = [CastlingFlags::ALL; 64];

    perms[precomputed::A1 as usize] = CastlingFlags::WQ.complement().intersection(CastlingFlags::ALL);
    perms[precomputed::H1 as usize] = CastlingFlags::WK.complement().intersection(CastlingFlags::ALL);
    perms[precomputed::A8 as usize] = CastlingFlags::BQ.complement().intersection(CastlingFlags::ALL);
    perms[precomputed::H8 as usize] = CastlingFlags::BK.complement().intersection(CastlingFlags::ALL);
    perms[precomputed::E1 as usize] = CastlingFlags::WHITE.complement().intersection(CastlingFlags::ALL);
    perms[precomputed::E8 as usize] = CastlingFlags::BLACK.complement().intersection(CastlingFlags::ALL);

    perms
}