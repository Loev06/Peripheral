use std::time::Instant;

use super::{Board, MoveGenerator};

mod search;
use search::TranspositionTable;

pub const MAX_DEPTH: usize = 64;
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
    search_canceled: bool
}

impl ChessEngine {
    pub fn new(fen: &str, table_size: usize) -> Self {
        Self {
            board: Board::try_from_fen(fen).expect("Invalid fen"),
            mg: MoveGenerator::new(),
            tt: TranspositionTable::new(table_size),

            timer: Instant::now(),
            nodes: 0,
            search_time: 0,
            search_canceled: false,
        }
    }

    pub fn reset_table(&mut self, size_mb: usize) {
        self.tt = TranspositionTable::new(size_mb);
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