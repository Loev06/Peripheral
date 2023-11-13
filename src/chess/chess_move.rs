use std::fmt;

use super::{PieceType, Square};

#[derive(Clone, Copy)]
pub struct Move {
    moving_piece: PieceType,
    capturing_piece: Option<PieceType>,
    from: Square,
    to: Square
}

impl Move {
    pub fn new(moving_piece: PieceType, from: Square, to: Square, capturing_piece: Option<PieceType>) -> Self {
        Self { moving_piece, capturing_piece, from, to }
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("move")
         .field(&self.moving_piece)
         .field(&self.from)
         .field(&self.to)
         .finish()
    }
}