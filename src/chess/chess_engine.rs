use std::time::Instant;

use super::{Board, MoveGenerator};

mod search;
use search::TranspositionTable;

const TT_ENTRY_SIZE: usize = 8;
const TT_SIZE_MB: usize = 8;
pub const TT_SIZE: usize = tt_size_from_mb(TT_SIZE_MB, TT_ENTRY_SIZE); // Must be a power of two
pub const TT_INDEX_SHIFT: usize = 64 - TT_SIZE.trailing_zeros() as usize;

pub const MAX_DEPTH: usize = 64;

const fn tt_size_from_mb(mb: usize, entry_size: usize) -> usize {
    let preferred_size = mb * 1024 * 1024 / entry_size;
    1 << preferred_size.ilog2() // round down
}

pub struct SearchParams {
    pub move_time: Option<u128>,
    pub wtime: u128,
    pub btime: u128,
    pub winc: u128,
    pub binc: u128,
    pub depth: u8
}

impl SearchParams {
    pub fn new() -> Self {
        Self {
            move_time: None,
            wtime: u128::MAX,
            btime: u128::MAX,
            winc: 0,
            binc: 0,
            depth: u8::MAX
        }
    }
}

pub struct ChessEngine {
    board: Board,
    mg: MoveGenerator,
    tt: TranspositionTable,

    timer: Instant,
    nodes: u64,
    search_time: u128,
    search_canceled: bool,

    current_position: u64
}

impl ChessEngine {
    pub fn new(fen: &str) -> Self {
        Self {
            board: Board::try_from_fen(fen).expect("Invalid fen"),
            mg: MoveGenerator::new(),
            tt: TranspositionTable::new(TT_SIZE),

            timer: Instant::now(),
            nodes: 0,
            search_time: 0,
            search_canceled: false,

            current_position: 0
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