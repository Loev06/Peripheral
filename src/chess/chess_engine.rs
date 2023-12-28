use super::{Board, MoveGenerator};

mod search;
pub struct ChessEngine {
    board: Board,
    mg: MoveGenerator
}

impl ChessEngine {
    pub fn new(fen: &str) -> Self {
        Self {
            board: Board::try_from_fen(fen).expect("Invalid fen"),
            mg: MoveGenerator::new()
        }
    }

    pub fn set_board(&mut self, fen: &str) {
        self.board = Board::try_from_fen(fen).expect("Invalid fen");
    }

    pub fn get_board_fen(&self) -> String {
        self.board.get_fen()
    }

    pub fn get_board_string(&self) -> String {
        self.board.to_string()
    }
}