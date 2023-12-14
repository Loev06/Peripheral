use std::mem;

use super::{
    GameState, zobrist::*,
    super::{
        Board, Move, PieceType, PieceType::*, precomputed, util
    }
};

const MOVE_HISTORY_CAPACITY: usize = 512;

#[derive(Clone, Copy)]
pub struct GSHistoryEntry {
    pub gs: GameState,
    pub captured_piece: Option<PieceType>
}

pub struct GSHistory {
    history: [GSHistoryEntry; MOVE_HISTORY_CAPACITY],
    count: usize
}

impl GSHistory {
    pub fn new() -> Self {
        Self {
            history: unsafe {
                let block: mem::MaybeUninit<[GSHistoryEntry; MOVE_HISTORY_CAPACITY]> = mem::MaybeUninit::uninit();
                block.assume_init()
            },
            count: 0
        }
    }

    pub fn push(&mut self, gs: GSHistoryEntry) {
        debug_assert!(self.count < MOVE_HISTORY_CAPACITY);
        self.history[self.count] = gs;
        self.count += 1;
    }

    pub fn pop(&mut self) -> GSHistoryEntry {
        debug_assert!(self.count > 0);
        self.count -= 1;
        self.history[self.count]
    }
}

impl Board {
    pub fn undo_move(&mut self, mv: &Move) {
        self.key ^= ZOBRIST_CASTLING[self.gs.castling_rights.bits() as usize];
        self.key ^= ZOBRIST_EP_SQUARE[util::ls1b_from_bitboard(self.gs.en_passant_mask) as usize];

        let gs_entry = self.gs_history.pop();
        self.gs = gs_entry.gs;
        
        let from = mv.get_from();
        let to = mv.get_to();
        let new_piece_type = self.piece_list[to as usize].expect("Moving piece should exist");
        let is_promotion = mv.is_promotion();
        
        if is_promotion { // Promotion
            self.remove_piece(new_piece_type, to);
            self.place_piece(PieceType::from_color(WPawn, self.gs.player_to_move), from);
        } else {
            self.move_piece(new_piece_type, to, from);

            if mv.intersects(Move::SPECIAL1) { // only set when promoting or castling, so must be castle
                match to {
                    precomputed::G1 => self.move_piece(WRook, precomputed::F1, precomputed::H1),
                    precomputed::C1 => self.move_piece(WRook, precomputed::D1, precomputed::A1),
                    precomputed::G8 => self.move_piece(BRook, precomputed::F8, precomputed::H8),
                    precomputed::C8 => self.move_piece(BRook, precomputed::D8, precomputed::A8),
                    sq => panic!("Invalid castling move to {}", sq)
                }
            } else if mv.is_ep() {
                self.place_piece(PieceType::from_color(WPawn, self.gs.opponent_color), to ^ 8);
            }
        }

        if let Some(pt) = gs_entry.captured_piece { // normal capture
            self.place_piece(pt, to);
        }
        
        self.key ^= ZOBRIST_BLACK_TO_MOVE; // Don't call switch_sides, as old gs containing switched data is loaded
        self.key ^= ZOBRIST_CASTLING[self.gs.castling_rights.bits() as usize];
        self.key ^= ZOBRIST_EP_SQUARE[util::ls1b_from_bitboard(self.gs.en_passant_mask) as usize];
        self.update_board_data();
    }
}
