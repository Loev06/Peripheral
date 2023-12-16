use std::{ops::{Neg, Add, Sub}, fmt::Display};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PieceType {
    WPawn    = 0,
    WKnight  = 1,
    WBishop  = 2,
    WRook    = 3,
    WQueen   = 4,
    WKing    = 5,

    BPawn    = 6,
    BKnight  = 7,
    BBishop  = 8,
    BRook    = 9,
    BQueen   = 10,
    BKing    = 11,

    AnyWhite = 12,
    AnyBlack = 13,
    AnyPiece = 14,

    WHVSlider = 15,
    BHVSlider = 16,
    WDSlider = 17,
    BDSlider = 18
}

impl PieceType {
    pub fn from_color(pt: PieceType, color: Color) -> PieceType {
        match color {
            Color::White => pt,
            Color::Black => match pt {
                PieceType::WPawn => PieceType::BPawn,
                PieceType::WKnight => PieceType::BKnight,
                PieceType::WBishop => PieceType::BBishop,
                PieceType::WRook => PieceType::BRook,
                PieceType::WQueen => PieceType::BQueen,
                PieceType::WKing => PieceType::BKing,

                PieceType::AnyWhite => PieceType::AnyBlack,
                PieceType::WHVSlider => PieceType::BHVSlider,
                PieceType::WDSlider => PieceType::BDSlider,
                
                pt => panic!("from_color expected a white piece, got: {}", pt)
            }
        }
    }
}

impl Add for PieceType {
    type Output = usize;

    fn add(self, rhs: Self) -> Self::Output {
        self as usize + rhs as usize
    }
}

impl Sub for PieceType {
    type Output = usize;

    fn sub(self, rhs: Self) -> Self::Output {
        self as usize - rhs as usize
    }
}

impl Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match *self {
            PieceType::WPawn    => "P",
            PieceType::WKnight  => "N",
            PieceType::WBishop  => "B",
            PieceType::WRook    => "R",
            PieceType::WQueen   => "Q",
            PieceType::WKing    => "K",
            PieceType::BPawn    => "p",
            PieceType::BKnight  => "n",
            PieceType::BBishop  => "b",
            PieceType::BRook    => "r",
            PieceType::BQueen   => "q",
            PieceType::BKing    => "k",
            pt => panic!("Not a valid PieceType: {}", pt as usize)
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    White = 1,
    Black = -1
}

impl Neg for Color {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            match self {
                Self::White => "w",
                Self::Black => "b"
            }
        )
    }
}