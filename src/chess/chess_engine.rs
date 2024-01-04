use super::{Board, MoveGenerator};

mod search;
use search::TranspositionTable;

const TT_ENTRY_SIZE: usize = 8;
const TT_SIZE_MB: usize = 8;
pub const TT_SIZE: usize = tt_size_from_mb(TT_SIZE_MB, TT_ENTRY_SIZE); // Must be a power of two
pub const TT_INDEX_SHIFT: usize = 64 - TT_SIZE.trailing_zeros() as usize;

const fn tt_size_from_mb(mb: usize, entry_size: usize) -> usize {
    let preferred_size = mb * 1024 * 1024 / entry_size;
    1 << preferred_size.ilog2() // round down
}

pub struct ChessEngine {
    board: Board,
    mg: MoveGenerator,
    tt: TranspositionTable
}

impl ChessEngine {
    pub fn new(fen: &str) -> Self {
        Self {
            board: Board::try_from_fen(fen).expect("Invalid fen"),
            mg: MoveGenerator::new(),
            tt: TranspositionTable::new(TT_SIZE)
        }
    }

    pub fn reset_table(&mut self) {
        self.tt = TranspositionTable::new(TT_SIZE);
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