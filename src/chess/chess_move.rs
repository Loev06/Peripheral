use std::{fmt, error::Error};
use bitflags::bitflags;

use crate::Board;

use super::{Square, precomputed, Color, PieceType::{self, *}};

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
    #[derive(Clone, Copy, PartialEq, Eq)]
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

        const KNIGHT_PROMOTION = Self::PROMOTION.bits();
        const BISHOP_PROMOTION = Self::PROMOTION.bits() | Self::SPECIAL0.bits();
        const ROOK_PROMOTION = Self::PROMOTION.bits() | Self::SPECIAL1.bits();
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

    pub fn get_from(&self) -> Square {
        (self.0.bits() & Self::FROM.bits()) as Square
    }
    pub fn get_to(&self) -> Square {
        ((self.0.bits() & Self::TO.bits()) >> 6) as Square
    }
    pub fn get_promotion_piece(&self, color: Color) -> PieceType {
        PieceType::from_color(
            match self.intersection(Self::QUEEN_PROMOTION) {
                Self::QUEEN_PROMOTION => WQueen,
                Self::KNIGHT_PROMOTION => WKnight,
                Self::ROOK_PROMOTION => WRook,
                Self::BISHOP_PROMOTION => WBishop,
                _ => panic!("Not a valid promotion move: {}", self)
            },
            color
        )
    }
    pub fn is_ep(&self) -> bool {
        self.contains(Move::EP_CAPTURE)
    }
    pub fn is_promotion(&self) -> bool {
        self.intersects(Move::PROMOTION)
    }

    pub fn try_from_str(name: &str, board: &Board) -> Result<Self, Box<dyn Error>> {
        let from = precomputed::SQUARE_NAMES.iter().position(|i| *i == &name[..2]).ok_or("Invalid from square")?;
        let to = precomputed::SQUARE_NAMES.iter().position(|i| *i == &name[2..4]).ok_or("Invalid to square")?;

        let mut mv = Self::new(from as Square, to as Square, &Self::empty());

        let diff = to as isize - from as isize;

        match board.piece_list[from] {
            Some(WPawn) | Some(BPawn) => {
                if diff.abs() == 16 {
                    mv.insert(Self::DOUBLE_PAWN_PUSH);
                } else if diff.abs() != 8 && board.piece_list[to] == None {
                    mv.insert(Self::EP_CAPTURE);
                }
            },
            Some(WKing) | Some(BKing) => {
                match diff {
                     2 => mv.insert(Self::KING_CASTLE),
                    -2 => mv.insert(Self::QUEEN_CASTLE),
                    _ => ()
                }
            },
            _ => ()
        }

        match name.chars().nth(4) {
            Some('q') => mv.insert(Self::QUEEN_PROMOTION),
            Some('r') => mv.insert(Self::ROOK_PROMOTION),
            Some('b') => mv.insert(Self::BISHOP_PROMOTION),
            Some('n') => mv.insert(Self::KNIGHT_PROMOTION),
            _ => ()
        }

        Ok(mv)
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}{}{} ({})",
            precomputed::SQUARE_NAMES[self.intersection(Move::FROM).bits() as usize],
            precomputed::SQUARE_NAMES[self.intersection(Move::TO).bits() as usize >> 6],
            SPECIAL_MOVE_NAMES[self.intersection(Move::SPECIAL_BITS).bits() as usize >> 12],
            self.0)
        )
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}{}{}",
            precomputed::SQUARE_NAMES[self.intersection(Move::FROM).bits() as usize],
            precomputed::SQUARE_NAMES[self.intersection(Move::TO).bits() as usize >> 6],
            match self.intersection(Move::SPECIAL_BITS) {
                Self::QUEEN_PROMOTION   => "q",
                Self::ROOK_PROMOTION    => "r",
                Self::BISHOP_PROMOTION  => "b",
                Self::KNIGHT_PROMOTION  => "n",
                _ => ""
            }
        ))
    }
}