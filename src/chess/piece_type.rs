use std::ops::Neg;

#[derive(Clone, Copy, Debug, enum_map::Enum, PartialEq)]
pub enum PieceType {
    Pawn(Color),
    Knight(Color),
    Bishop(Color),
    Rook(Color),
    Queen(Color),
    King(Color),

    Any(Color),
    HVslider(Color),
    Dslider(Color)
}

#[derive(Clone, Copy, Debug, enum_map::Enum, PartialEq)]
pub enum Color {
    White = 1,
    Black = -1,
    Neutral = 0
}

impl Neg for Color {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
            Color::Neutral => Color::Neutral
        }
    }
}