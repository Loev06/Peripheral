use std::mem;

use super::{super::{Board, Move, PieceType, PieceType::*, precomputed}, GameState};

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
        
        self.update_board_data();
    }
}

// struct OldSquareData {
//     piece: Option<PieceType>,
//     square: Square
// }

// impl OldSquareData {
//     fn from_square(sq: Square, b: &Board) -> Self {
//         Self {
//             piece: b.piece_list[sq as usize],
//             square: sq
//         }
//     }
// }

// pub struct UndoMoveData {
//     from: OldSquareData,
//     to: OldSquareData,
//     old_ep_mask: Bitboard,
//     old_castling_flags: u8
// }

// impl UndoMoveData {
//     pub fn new(from: Square, to: Square, b: &Board) -> Self {
//         Self {
//             from: OldSquareData::from_square(from, b),
//             to: OldSquareData::from_square(to, b),
//             old_ep_mask: b.en_passant_mask,
//             old_castling_flags: b.castling_rights.bits()
//         }
//     }

//     pub fn undo_move(&self, b: &mut Board) {
//         let from = self.from;
//         let to = self.to;
//         b.piece_list[from.square as usize] = from.piece;
//         b.piece_list[to.square as usize] = to.piece;

//         let moving_piece_diffs = util::bitboard_from_square(from.square) | util::bitboard_from_square(to.square);
//         b.bbs[from.piece] ^= moving_piece_diffs;
//         if let Some(pt) = to.piece {
//             b.bbs[pt] ^= util::bitboard_from_square(to.square);
//         }

//         b.en_passant_mask = self.old_ep_mask;
//         b.castling_rights = CastlingFlags::new(self.old_castling_flags);
//     }
// }