use std::{error::Error, time::Instant};

use super::{Board, MoveGenerator, Move};

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
    search_canceled: bool,

    uci_moves: Vec<Move>
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
            
            uci_moves: Vec::new()
        }
    }

    pub fn reset_table(&mut self, size_mb: usize) {
        self.tt = TranspositionTable::new(size_mb);
    }

    pub fn set_board(&mut self, fen: &str) -> Result<(), Box<dyn Error>> {
        self.board = Board::try_from_fen(fen)?;
        Ok(())
    }

    pub fn get_board(&self) -> &Board {
        &self.board
    }

    pub fn make_uci_move(&mut self, mv: &str) -> Result<(), Box<dyn Error>> {
        let mv = Move::try_from_str(mv, &self.board)?;
        self.board.make_move(&mv);
        self.uci_moves.push(mv);
        Ok(())
    }

    pub fn undo_move(&mut self) -> Result<(), &str>{
        self.board.undo_move(&self.uci_moves.pop().ok_or("No move to undo")?);
        Ok(())
    }
}