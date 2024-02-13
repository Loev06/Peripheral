use super::{
    zobrist::*,
    super::{
        Board, Move, PieceType, Bitboard, PieceType::*, precomputed, util
    }
};

impl Board {
    pub fn undo_move(&mut self, mv: &Move) {
        self.key ^= ZOBRIST_CASTLING[self.gs.castling_rights.bits() as usize];
        self.key ^= ZOBRIST_EP_SQUARE[util::ls1b_from_bitboard(self.gs.en_passant_mask) as usize];

        let gs_entry = self.gs_history.pop();
        self.gs = gs_entry.gs;

        self.key_history.pop();
        
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

    #[inline(always)]
    pub fn undo_null_move(&mut self, ep_mask: Bitboard) {
        self.switch_sides();
        self.gs.playing_king_square = util::ls1b_from_bitboard(self.bbs[WKing + self.gs.pt_offset]);
        
        self.gs.en_passant_mask = ep_mask;
    }
}
