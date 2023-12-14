use super::{Board, MoveGenerator};

mod search;
pub struct ChessEngine {
    board: Board,
    mg: MoveGenerator
}

impl ChessEngine {
    pub fn new() -> Self {
        Self {
            board: Board::empty(),
            mg: MoveGenerator::new()
        }
    }
}